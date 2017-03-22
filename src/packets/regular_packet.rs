use frames::quic_frame::QuicFrame;

#[derive(Debug, Clone)]
pub struct RegularPacket {
    pub frames: Vec<QuicFrame>,
}