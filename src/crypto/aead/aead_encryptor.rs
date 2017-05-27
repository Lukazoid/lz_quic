use errors::*;
use packets::PacketNumber;

pub trait AeadEncryptor {
    fn encrypt(&mut self, associated_data: &[u8], plain_text: &[u8], packet_number: PacketNumber) -> Result<Vec<u8>>;
}