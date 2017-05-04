use errors::*;

pub trait KeyExchange {
    fn public_key(&self) -> &[u8];

    fn calculate_shared_key(self, other_public_key: &[u8]) -> Result<Vec<u8>>;
}