use bytes::{Bytes, BytesMut};
use debugit::DebugIt;
use errors::*;
use frames::Frame;
use packets::PacketNumber;
use protocol::{ConnectionId, Readable, Writable};
use ring::aead::{self, OpeningKey, SealingKey};
use ring::digest;
use ring::hkdf;
use ring::hmac::SigningKey;
use rustls::Session;
use smallvec::SmallVec;
use std::mem;

#[derive(Debug)]
pub struct CryptoState {
    secret: DebugIt<SigningKey>,
    sealing_key: DebugIt<SealingKey>,
    opening_key: DebugIt<OpeningKey>,
    iv: SmallVec<[u8; digest::MAX_OUTPUT_LEN]>,
}

static HANDSHAKE_SALT: [u8; 20] = [
    0x9c, 0x10, 0x8f, 0x98, 0x52, 0x0a, 0x5c, 0x5c, 0x32, 0x96, 0x8e, 0x95, 0x0e, 0x8a, 0x2c, 0x5f,
    0xe0, 0x6d, 0x6c, 0x38,
];

impl CryptoState {
    pub fn for_handshake(
        destination_connection_id: ConnectionId,
        label: &str,
    ) -> Result<CryptoState> {
        // The hash function for HKDF when deriving handshake secrets and keys
        // is SHA-256
        let hash_algorithm = &digest::SHA256;

        // Prior to establishing a shared secret, packets are protected with
        // AEAD_AES_128_GCM and a key derived from the client’s connection ID
        let aead_algorithm = &aead::AES_128_GCM;

        let salt = SigningKey::new(hash_algorithm, &HANDSHAKE_SALT[..]);
        let handshake_secret = hkdf::extract(&salt, destination_connection_id.bytes());

        let our_handshake_secret =
            qhkdf_expand(&handshake_secret, label, hash_algorithm.output_len)?;
        let signing_key = SigningKey::new(hash_algorithm, &our_handshake_secret[..]);

        Self::new(signing_key, aead_algorithm)
    }

    pub fn from_tls<S: Session>(session: &S, label: &str) -> Result<CryptoState> {
        let supported_cipher_suite = session
            .get_negotiated_ciphersuite()
            .ok_or_else(|| ErrorKind::FailedToExportTlsKeyingMaterial)?;

        let hash_algorithm = supported_cipher_suite.get_hash();
        let mut secret: SmallVec<[u8; digest::MAX_OUTPUT_LEN]> =
            smallvec![0; hash_algorithm.output_len];
        session
            .export_keying_material(&mut secret, label.as_bytes(), None)
            .chain_err(|| ErrorKind::FailedToExportTlsKeyingMaterial)?;

        let secret = SigningKey::new(hash_algorithm, &secret[..]);
        Self::new(secret, supported_cipher_suite.get_aead_alg())
    }

    fn new(secret: SigningKey, aead_algorithm: &'static aead::Algorithm) -> Result<CryptoState> {
        let key = qhkdf_expand(&secret, "key", aead_algorithm.key_len())?;

        // As defined in Section 5.3 of [TLS13], the IV length is the larger of
        // 8 or N_MIN (see Section 4 of [AEAD]; all ciphersuites defined in
        // [TLS13] have N_MIN set to 12)
        let iv = qhkdf_expand(&secret, "iv", 12)?;

        // TODO LH pn_key = QHKDF-Expand(S, "pn", pn_key_length)

        let sealing_key = SealingKey::new(aead_algorithm, &key[..])
            .chain_err(|| ErrorKind::FailedToBuildCryptoState)?;
        let opening_key = OpeningKey::new(aead_algorithm, &key[..])
            .chain_err(|| ErrorKind::FailedToBuildCryptoState)?;

        let crypto_state = CryptoState {
            secret: DebugIt(secret),
            sealing_key: DebugIt(sealing_key),
            opening_key: DebugIt(opening_key),
            iv,
        };

        Ok(crypto_state)
    }

    pub fn with_key_update(&self, label: &str) -> Result<CryptoState> {
        let secret = &self.secret.0;

        let hash_algorithm = secret.digest_algorithm();
        let new_secret = qhkdf_expand(secret, label, hash_algorithm.output_len)?;

        let new_secret = SigningKey::new(hash_algorithm, &new_secret[..]);
        Self::new(new_secret, self.opening_key.0.algorithm())
    }

    fn make_nonce(
        &self,
        packet_number: PacketNumber,
    ) -> Result<SmallVec<[u8; digest::MAX_OUTPUT_LEN]>> {
        // The nonce, N, is formed by combining the packet protection IV
        // (either client_pp_iv<i> or server_pp_iv<i>) with the packet number.
        // The 64 bits of the reconstructed QUIC packet number in network byte
        // order is left-padded with zeros to the size of the IV. The exclusive
        // OR of the padded packet number and the IV forms the AEAD nonce.

        let mut nonce = self.iv.clone();

        let packet_number_int: u64 = packet_number.into();
        let packet_number_bytes: SmallVec<[u8; mem::size_of::<u64>()]> =
            packet_number_int.bytes_small()?;
        assert!(nonce.len() >= packet_number_bytes.len());

        // skip the left-padding bytes
        let skip_nonce_bytes = nonce.len() - packet_number_bytes.len();

        for (nonce, pn) in nonce
            .iter_mut()
            .skip(skip_nonce_bytes)
            .zip(packet_number_bytes)
        {
            *nonce ^= pn;
        }

        Ok(nonce)
    }

    fn sealing_key(&self) -> &SealingKey {
        &self.sealing_key.0
    }

    fn opening_key(&self) -> &OpeningKey {
        &self.opening_key.0
    }

    pub fn seal(
        &self,
        packet_number: PacketNumber,
        packet_header_bytes: &[u8],
        frames: &[Frame],
    ) -> Result<Bytes> {
        let nonce = self.make_nonce(packet_number)?;
        let sealing_key = self.sealing_key();

        let mut in_out = frames.bytes()?;

        let tag_len = sealing_key.algorithm().tag_len();

        let in_out_len = in_out.len();
        in_out.resize(in_out_len + tag_len, 0);

        let out_len = aead::seal_in_place(
            sealing_key,
            &nonce[..],
            packet_header_bytes,
            &mut in_out,
            tag_len,
        ).chain_err(|| ErrorKind::FailedToSealData)?;

        Ok((&in_out[..out_len]).into())
    }

    pub fn open(
        &self,
        packet_number: PacketNumber,
        packet_header_bytes: &[u8],
        ciphertext: &[u8],
    ) -> Result<Vec<Frame>> {
        let nonce = self.make_nonce(packet_number)?;
        let opening_key = self.opening_key();

        let mut ciphertext: BytesMut = ciphertext.into();

        let plaintext = aead::open_in_place(
            opening_key,
            &nonce[..],
            packet_header_bytes,
            0,
            &mut ciphertext[..],
        ).chain_err(|| ErrorKind::FailedToOpenSealedData)?;

        Ok(Readable::collect_from_bytes(plaintext)?)
    }
}

fn encode_hkdf_info(label: &str, out_len: usize) -> Result<SmallVec<[u8; digest::MAX_OUTPUT_LEN]>> {
    // struct {
    //     uint16 length = Length;
    //     opaque label<6..255> = "QUIC " + Label;
    // } QhkdfExpandInfo;

    let mut info: SmallVec<[u8; 64]> = smallvec![];
    (out_len as u16).write_to_small_vec(&mut info)?;

    let label_prefix = b"QUIC ";
    let label_len = label_prefix.len() + label.len();

    assert!(label_len <= u8::max_value() as usize);

    (label_len as u8).write_to_small_vec(&mut info)?;

    label_prefix.write_to_small_vec(&mut info)?;
    label.as_bytes().write_to_small_vec(&mut info)?;

    Ok(info)
}

fn qhkdf_expand(
    signing_key: &SigningKey,
    label: &str,
    out_len: usize,
) -> Result<SmallVec<[u8; 64]>> {
    let info = encode_hkdf_info(label, out_len)?;

    let mut out = smallvec![0; out_len];
    hkdf::expand(&signing_key, &info[..], &mut out[..]);

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use frames::Frame;
    use protocol::ConnectionId;
    use rand;
    use ring::digest;
    use ring::hmac::SigningKey;

    #[test]
    fn encode_hkdf_info_encodes_correctly() {
        let info = encode_hkdf_info("key", 32).unwrap();

        assert_eq!(
            [
                0x00, 0x20, 0x08, 0x51, 0x55, 0x49, 0x43, 0x20, 0x6b, 0x65, 0x79
            ],
            &info[..]
        );
    }

    #[test]
    fn qhkdf_expand_works() {
        let algorithm = &digest::SHA256;

        qhkdf_expand(
            &SigningKey::new(algorithm, b"some secret"),
            "key",
            algorithm.output_len,
        ).unwrap();
    }

    #[test]
    fn crypto_state_for_handshake_works() {
        CryptoState::for_handshake(
            ConnectionId::generate(&mut rand::thread_rng()),
            "my test label",
        ).unwrap();
    }

    #[test]
    fn crypto_state_seal_open() {
        let crypto_state = CryptoState::for_handshake(
            ConnectionId::generate(&mut rand::thread_rng()),
            "my test label",
        ).unwrap();

        let packet_number = 1254u32.into();
        let packet_header_bytes = b"some packet header bytes";

        let original_frames = vec![Frame::Padding, Frame::Ping, Frame::Padding];
        let sealed = crypto_state
            .seal(packet_number, packet_header_bytes, &original_frames[..])
            .unwrap();

        let opened_frames = crypto_state
            .open(packet_number, packet_header_bytes, &sealed[..])
            .unwrap();

        assert_eq!(opened_frames, original_frames);
    }
}
