use frames::frame::Frame;

#[derive(Debug, Clone)]
pub struct RegularPacket {
    pub frames: Vec<Frame>,
}