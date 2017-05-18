use errors::*;

pub trait AeadEncryptor {
    fn encrypt(&mut self, associated_data: &[u8], plain_text: &[u8]) -> Vec<u8>;
}
        // TODO LH Add AEAD support
        // use https://docs.rs/openssl-sys/0.9.11/openssl_sys/fn.EVP_CIPHER_CTX_ctrl.html with https://docs.rs/openssl-sys/0.9.11/openssl_sys/constant.EVP_CTRL_GCM_SET_TAG.html to set the tag length