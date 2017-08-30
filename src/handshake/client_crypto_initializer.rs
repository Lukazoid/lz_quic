use errors::*;
use crypto::aead::{AeadDecryptor, AeadEncryptor, AesGcmDecryptor, AesGcmEncryptor, NullAeadDecryptor};
use crypto::key_derivation::{Sha256HkdfKeyDeriver, KeyDeriver, DerivedKeys};
use crypto::certificates::{Certificate, CertificateManager};
use crypto::DiversificationNonce;
use protocol::{ConnectionId, Perspective, EncryptionLevel, Writable};
use handshake::{ServerConfiguration, HandshakeMessage, ClientHelloMessage, ServerHelloMessage, RejectionMessage,
    ClientNonce, ServerNonce, SourceAddressToken};
use packets::PacketNumber;
use handshake::CryptoStage;

#[derive(Debug, Default)]
struct CachedServerInformation {
    source_address_token: Option<SourceAddressToken>,
    server_nonce: Option<ServerNonce>,
    server_configuration: Option<ServerConfiguration>,
}

#[derive(Debug)]
pub struct ClientCryptoInitializer {
    connection_id: ConnectionId,
    crypto_stage: CryptoStage,
    certificate_manager: CertificateManager,
    last_sent_client_hello: Option<ClientHelloMessage>,
    cached_server_information: CachedServerInformation,
}

impl ClientCryptoInitializer {
    pub fn new(connection_id: ConnectionId) -> Self {
        // TODO LH Eventually we want to pass the TLS configuration through and use it to initialize the
        // certificate_manager

        Self {
            connection_id: connection_id,
            crypto_stage: CryptoStage::Unencrypted,
            certificate_manager: CertificateManager::skip_verify(),
            last_sent_client_hello: None,
            cached_server_information: Default::default(),
        }
    }

    pub fn open(&self, associated_data: &[u8], raw: &[u8], packet_number: PacketNumber) -> Result<(EncryptionLevel, Vec<u8>)> {
        self.crypto_stage.decrypt(associated_data, raw, packet_number)
    }

    pub fn seal(&self, associated_data: &[u8], raw: &[u8], packet_number: PacketNumber) -> Result<(EncryptionLevel, Vec<u8>)> {
        self.crypto_stage.encrypt(associated_data, raw, packet_number)
    }

    fn generate_client_nonce(&self) -> Result<ClientNonce> {
        let server_orbit = self.cached_server_information.server_configuration.as_ref().map(|conf|conf.orbit)
            .ok_or_else(||Error::from(ErrorKind::NoOrbitForClientNonce))?;

        ClientNonce::generate(server_orbit)
    }

    fn handle_server_hello_message(&mut self, server_hello_message: ServerHelloMessage, message_encryption_level: EncryptionLevel) -> Result<()> {

        // Only handle encrypted server hello messages
        if !matches!(message_encryption_level, EncryptionLevel::NonForwardSecure) {
            bail!(ErrorKind::ReceivedUnencryptedServerHello);
        }

        if let Some(server_configuration) = server_hello_message.server_configuration {
            if server_configuration.is_expired() {
                bail!(ErrorKind::ServerConfigurationExpired);
            }

            self.cached_server_information.server_configuration = Some(server_configuration);

        }

// self.derive_keys(true, server_hello_message.)

        Ok(())
    }

    fn handle_rejection_message(&mut self, rejection_message: RejectionMessage, message_encryption_level: EncryptionLevel) -> Result<()> {
        if let Some(server_configuration) = rejection_message.server_configuration {
            if server_configuration.is_expired() {
                bail!(ErrorKind::ServerConfigurationExpired);
            }

            self.cached_server_information.server_configuration = Some(server_configuration);
        }

        

        Ok(())
    }

    pub fn handle_handshake_message(&mut self, handshake_message: HandshakeMessage, message_encryption_level: EncryptionLevel) -> Result<()> {
        match handshake_message {
            HandshakeMessage::ServerHello(server_hello_message) => {
                self.handle_server_hello_message(server_hello_message, message_encryption_level)
            },
            HandshakeMessage::Rejection(rejection_message) => {
                self.handle_rejection_message(rejection_message, message_encryption_level)
            },
            _ => {
                Err(ErrorKind::InvalidCryptoMessageType(handshake_message.tag()).into())
            }
        }
    }

    fn derive_keys(
        &self,
        forward_secure: bool,
        nonce: &[u8],
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

        key_deriver.derive_keys(&server_configuration.shared_key, 
            nonce, 
            last_sent_chlo, 
            server_configuration,
            leaf_certificate,
            diversification_nonce)
    }
}