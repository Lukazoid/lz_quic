use errors::*;
use packets::PacketNumber;

pub trait AeadDecryptor {
    fn decrypt(&mut self, associated_data: &[u8], cipher_text: &[u8], packet_number: PacketNumber) -> Result<Vec<u8>>;
}