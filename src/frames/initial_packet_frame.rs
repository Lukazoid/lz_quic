use frames::{AckFrame, CryptoFrame};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum InitialPacketFrame {
    Crypto(CryptoFrame),
    Ack(AckFrame),
}
