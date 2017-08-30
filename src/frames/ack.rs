use primitives::U48;
use chrono::Duration;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LargestObservedFieldLength {
    One = 1,
    Two = 2,
    Four = 4,
    Six = 6,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MissingPacketSequenceNumberDeltaFieldLength {
    One = 1,
    Two = 2,
    Four = 4,
    Six = 6,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AckFrameTypeProperties {
    pub multiple_ack_range: bool,
    pub largest_observed_field_length: LargestObservedFieldLength,
    pub missing_packet_sequence_number_delta_field_length:MissingPacketSequenceNumberDeltaFieldLength
}

bitflags!(
    flags AckFrameTypeFlags : u8 {
        const MULTIPLE_ACK_RANGE                = 0b00100000,
        const LARGEST_FIELD_LENGTH_TWO          = 0b00000100,
        const LARGEST_FIELD_LENGTH_FOUR         = 0b00001000,
        const LARGEST_FIELD_LENGTH_SIX          = 0b00001100,
        const MISSING_PACKET_DELTA_LENGTH_TWO   = 0b00000001,
        const MISSING_PACKET_DELTA_LENGTH_FOUR  = 0b00000010,
        const MISSING_PACKET_DELTA_LENGTH_SIX   = 0b00000011,
    }
);

impl From<AckFrameTypeProperties> for AckFrameTypeFlags {
    fn from(value: AckFrameTypeProperties) -> AckFrameTypeFlags {
        let mut ack_frame_type_flags = AckFrameTypeFlags::empty();

        if value.multiple_ack_range {
            ack_frame_type_flags.insert(MULTIPLE_ACK_RANGE);
        }

        let largest_observed_field_length_flag = match value.largest_observed_field_length {
            LargestObservedFieldLength::One => AckFrameTypeFlags::empty(),
            LargestObservedFieldLength::Two => LARGEST_FIELD_LENGTH_TWO,
            LargestObservedFieldLength::Four => LARGEST_FIELD_LENGTH_FOUR,
            LargestObservedFieldLength::Six => LARGEST_FIELD_LENGTH_SIX,
        };

        ack_frame_type_flags.insert(largest_observed_field_length_flag);

        let missing_packet_delta_length_flags =
            match value.missing_packet_sequence_number_delta_field_length {
                MissingPacketSequenceNumberDeltaFieldLength::One => AckFrameTypeFlags::empty(),
                MissingPacketSequenceNumberDeltaFieldLength::Two => MISSING_PACKET_DELTA_LENGTH_TWO,
                MissingPacketSequenceNumberDeltaFieldLength::Four => {
                    MISSING_PACKET_DELTA_LENGTH_FOUR
                }
                MissingPacketSequenceNumberDeltaFieldLength::Six => MISSING_PACKET_DELTA_LENGTH_SIX,
            };

        ack_frame_type_flags.insert(missing_packet_delta_length_flags);

        ack_frame_type_flags
    }
}

impl From<AckFrameTypeProperties> for u8 {
    fn from(value: AckFrameTypeProperties) -> u8 {
        AckFrameTypeFlags::from(value).bits()
    }
}

#[derive(Debug, Clone)]
pub struct AckFrame {
    pub multiple_ack_range: bool,
    pub largest_acked_packet_number: U48,
    pub largest_acked_delta_time: Duration,
}

#[derive(Debug, Clone)]
pub struct AckBlock {

}

#[derive(Debug, Clone)]
pub struct AckTimestampBlock {

}