#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LongHeaderPacketType {
    Initial,
    Retry,
    Handshake,
    ZeroRttProtected,
}
