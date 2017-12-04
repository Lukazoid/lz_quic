use errors::*;
use protocol::EncryptionLevel;
use packets::PacketNumber;

#[derive(Debug)]
pub struct ServerCryptoInitializer {}

impl ServerCryptoInitializer {
    pub fn open(
        &self,
        associated_data: &[u8],
        raw: &[u8],
        packet_number: PacketNumber,
    ) -> Result<(EncryptionLevel, Vec<u8>)> {
        unimplemented!()
    }

    pub fn seal(
        &self,
        associated_data: &[u8],
        raw: &[u8],
        packet_number: PacketNumber,
    ) -> Result<(EncryptionLevel, Vec<u8>)> {
        unimplemented!()
    }
}
