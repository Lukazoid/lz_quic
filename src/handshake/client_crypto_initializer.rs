use errors::*;
use {DataStream, ClientPerspective};
use crypto::aead::{AesGcmEncryptor, NullAeadDecryptor};
use crypto::key_derivation::{Sha256HkdfKeyDeriver, KeyDeriver, DerivedKeys};
use crypto::certificates::CertificateManager;
use crypto::signing::Signature;
use crypto::{DiversificationNonce, Proof};
use protocol::{ConnectionId, Perspective, EncryptionLevel};
use protocol::version;
use handshake::{ServerConfiguration, HandshakeMessage, ClientHelloMessage, ServerHelloMessage, RejectionMessage,
    ClientNonce, ServerNonce, SourceAddressToken, HandshakeCodec};
use futures::{Async, Future, Poll};
use futures::stream::{self, Stream};
use futures::sink::Sink;
use handshake::CryptoStage;
use rand::OsRng;
use std::sync::{Arc, RwLock};
use smallvec::SmallVec;
use tokio_io::AsyncRead;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

#[derive(Debug, Default)]
struct CachedServerInformation {
    source_address_token: Option<SourceAddressToken>,
    server_nonce: Option<ServerNonce>,
    server_configuration: Option<ServerConfiguration>,
    server_proof: Option<(Signature, Arc<ClientHelloMessage>)>,
}

/// A `Stream` which will only take items until forward-secure encryption has been established.
#[derive(Debug)]
struct TakeUntilForwardSecureEncryption<S> {
    crypto_stage: Arc<RwLock<CryptoStage>>,
    stream: S,
}

impl<S: Stream> Stream for TakeUntilForwardSecureEncryption<S> {
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let crypto_stage = self.crypto_stage.read().unwrap();
        if matches!(crypto_stage.encryption_level(), EncryptionLevel::ForwardSecure) {
            return Ok(Async::Ready(None));
        }

        self.stream.poll()
    }
}

#[derive(Debug)]
pub struct ClientCryptoInitializer {
    hostname: String,
    connection_id: ConnectionId,
    crypto_stage: Arc<RwLock<CryptoStage>>,
    certificate_manager: CertificateManager,
    last_sent_client_hello: Option<Arc<ClientHelloMessage>>,
    cached_server_information: CachedServerInformation,
    client_nonce: Option<ClientNonce>,
    has_verified_server: bool,
}

impl ClientCryptoInitializer {
    pub fn new<H: Into<String>>(hostname: H, connection_id: ConnectionId) -> Self {
        // TODO LH Eventually we want to pass the TLS configuration through and use it to initialize the
        // certificate_manager

        Self {
            hostname: hostname.into(),
            connection_id: connection_id,
            crypto_stage: Arc::new(RwLock::new(CryptoStage::Unencrypted)),
            certificate_manager: CertificateManager::skip_verify(),
            last_sent_client_hello: None,
            cached_server_information: Default::default(),
            client_nonce: None,
            has_verified_server: false,
        }
    }

    pub fn crypto_stage(&self) -> &Arc<RwLock<CryptoStage>> {
        &self.crypto_stage
    }

    fn generate_client_nonce(&mut self, server_configuration: &ServerConfiguration) -> Result<()> {
        if self.client_nonce.is_some() {
            bail!(ErrorKind::TheClientNonceHasAlreadyBeenGenerated);
        }

        let server_orbit = server_configuration.orbit;
        
        let mut rng = OsRng::new()
            .chain_err(|| {
                ErrorKind::FailedToCreateCryptographicRandomNumberGenerator
            })?;

        self.client_nonce = Some(ClientNonce::generate(&mut rng, server_orbit)?);

        Ok(())
    }

    fn cache_source_address_token(&mut self, source_address_token: Option<SourceAddressToken>) {
        if let Some(source_address_token) = source_address_token {
            self.cached_server_information.source_address_token = Some(source_address_token);
        }
    }

    fn cache_server_nonce(&mut self, server_nonce: Option<ServerNonce>) {
        if let Some(server_nonce) = server_nonce {
            self.cached_server_information.server_nonce = Some(server_nonce);
        }
    }

    fn cache_server_configuration(&mut self, server_configuration: Option<ServerConfiguration>) -> Result<()> {
        if let Some(server_configuration) = server_configuration {
            if server_configuration.is_expired() {
                bail!(ErrorKind::ServerConfigurationExpired);
            }

            self.generate_client_nonce(&server_configuration)?;

            self.cached_server_information.server_configuration = Some(server_configuration);
        }

        Ok(())
    }

    fn cache_server_proof(&mut self, server_proof: Option<Signature>) -> Result<()> {
        if let Some(server_proof) = server_proof {

            let last_sent_client_hello = self.last_sent_client_hello
                .as_ref()
                .ok_or_else(|| Error::from(ErrorKind::ServerProofProvidedBeforeClientHelloSent))?;

            self.cached_server_information.server_proof = Some((server_proof, last_sent_client_hello.clone()));
        }

        Ok(())
    }

    fn set_certificates(&mut self, compressed_certificate_chain: Option<Vec<u8>>) -> Result<()>{
        if let Some(compressed_certificate_chain) = compressed_certificate_chain {
            self.certificate_manager.set_data(compressed_certificate_chain.as_slice())?;

            self.certificate_manager.verify(&self.hostname)?;

            self.has_verified_server = true;
        }

        Ok(())
    }

    fn has_decrypted_packet(&self) -> bool {
        let crypto_stage = self.crypto_stage.read().unwrap();
        crypto_stage.has_decrypted_packet()
    }

    fn handle_server_hello_message(&mut self, server_hello_message: ServerHelloMessage) -> Result<()> {
        // Only handle encrypted server hello messages
        if !self.has_decrypted_packet() {
            bail!(ErrorKind::ReceivedUnencryptedServerHello);
        }

        self.cache_source_address_token(server_hello_message.source_address_token);
        self.cache_server_nonce(server_hello_message.server_nonce);
        self.cache_server_configuration(server_hello_message.server_configuration)?;
        self.cache_server_proof(server_hello_message.server_proof)?;
        self.set_certificates(server_hello_message.compressed_certificate_chain)?;

        let derived_keys = self.derive_keys(true, None)?;

        let mut crypto_stage = self.crypto_stage.write().unwrap();
        crypto_stage.upgrade_to_forward_secure(derived_keys)?;

        Ok(())
    }

    fn handle_rejection_message(&mut self, rejection_message: RejectionMessage) -> Result<()> {
        self.cache_source_address_token(rejection_message.source_address_token);
        self.cache_server_nonce(rejection_message.server_nonce);
        self.cache_server_configuration(rejection_message.server_configuration)?;
        self.cache_server_proof(rejection_message.server_proof)?;
        self.set_certificates(rejection_message.compressed_certificate_chain)?;

        Ok(())
    }

    pub fn handle_handshake_message(&mut self, handshake_message: HandshakeMessage) -> Result<()> {
        match handshake_message {
            HandshakeMessage::ServerHello(server_hello_message) => {
                self.handle_server_hello_message(server_hello_message)
            },
            HandshakeMessage::Rejection(rejection_message) => {
                self.handle_rejection_message(rejection_message)
            },
            _ => {
                bail!(ErrorKind::InvalidCryptoMessageType(handshake_message.tag()));
            }
        }
    }

    pub fn client_hello(&self) -> ClientHelloMessage {
        ClientHelloMessage {
            server_name: Some(self.hostname.clone()),
            source_address_token: self.cached_server_information.source_address_token.clone(),
            proof_demands: SmallVec::from_buf([Proof::X509]),
            common_certificate_sets: self.certificate_manager.common_certificate_set_hashes(),
            cached_certificates: Vec::with_capacity(0),
            version: version::DRAFT_IETF_01,
            leaf_certificate: self.certificate_manager.leaf_certificate_hash(),
        }
    }

    pub fn handshake(mut self, crypto_stream: DataStream<ClientPerspective>) -> Box<Future<Item=(), Error=Error> + Send> {
        let (crypto_sink, crypto_stream) = crypto_stream.framed(HandshakeCodec::default()).split();

        let crypto_stage = self.crypto_stage().clone();

        let inchoate_client_hello = self.client_hello();
            
        let client_hellos = stream::once(Ok(HandshakeMessage::ClientHello(inchoate_client_hello)))
            .chain(crypto_stream
                .and_then(move |handshake_message| {
                    self.handle_handshake_message(handshake_message)
                        .map_err(|e| {
                            IoError::new(IoErrorKind::InvalidData, format!("{}", e))
                        })?;

                    let client_hello = self.client_hello();

                    Ok(HandshakeMessage::ClientHello(client_hello))
                }));

        let client_hellos = TakeUntilForwardSecureEncryption {
            crypto_stage: crypto_stage,
            stream: client_hellos
        };

        let future = crypto_sink.send_all(client_hellos)
            .chain_err(|| ErrorKind::FailedToPerformClientHandshake)
            .map(|_| ());

        Box::new(future)
    }

    fn derive_keys(
        &self,
        forward_secure: bool,
        diversification_nonce: Option<&DiversificationNonce>,
    ) -> Result<DerivedKeys> {

        let server_configuration = self.cached_server_information.server_configuration.as_ref().ok_or_else(|| {   
            Error::from(ErrorKind::ServerConfigurationIsRequiredBeforeForwardSecureEncryptionCanBeEstablished)
        })?;

        let last_sent_chlo = self.last_sent_client_hello.as_ref().ok_or_else(|| {
            Error::from(ErrorKind::ServerConfigurationIsRequiredBeforeForwardSecureEncryptionCanBeEstablished)
        })?;

        let leaf_certificate = self.certificate_manager.leaf_certificate().ok_or_else(|| {
            Error::from(ErrorKind::UnableToDeriveKeysWithoutALeafCertificate)
        })?;

        let key_deriver = Sha256HkdfKeyDeriver::new(forward_secure, Perspective::Client, self.connection_id, 16);

        let client_nonce = self.client_nonce.as_ref().ok_or_else(|| {
            Error::from(ErrorKind::ClientNonceIsRequiredBeforeForwardSecureEncryptionCanBeEstablished)
        })?;

        let server_nonce = self.cached_server_information.server_nonce.as_ref().ok_or_else(|| {
            Error::from(ErrorKind::ServerNonceIsRequiredBeforeForwardSecureEncryptionCanBeEstablished)
        })?;

        let nonce = [client_nonce.bytes(), server_nonce.bytes()].concat();

        key_deriver.derive_keys(&server_configuration.shared_key, 
            nonce.as_slice(), 
            last_sent_chlo, 
            server_configuration,
            leaf_certificate,
            diversification_nonce)
    }
}