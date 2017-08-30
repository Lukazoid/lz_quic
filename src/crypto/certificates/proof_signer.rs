use errors::*;

pub trait ProofSigner {
    fn sign(&self, data: &[u8]) -> Result<Vec<u8>>;
}