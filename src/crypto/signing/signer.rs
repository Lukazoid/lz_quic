use errors::*;
use crypto::signing::Signature;

pub trait Signer {
    fn sign(&self, data: &[u8]) -> Result<Signature>;
}