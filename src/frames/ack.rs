use chrono::Duration;

impl From<AckFrameTypeProperties> for u8 {
    fn from(value: AckFrameTypeProperties) -> u8 {
        AckFrameTypeFlags::from(value).bits()
    }
}

#[derive(Debug, Clone)]
pub struct AckFrame {
    pub multiple_ack_range: bool,
    pub largest_acked_packet_number: u64,
    pub largest_acked_delta_time: Duration,
}

#[derive(Debug, Clone)]
pub struct AckBlock {}

#[derive(Debug, Clone)]
pub struct AckTimestampBlock {}
