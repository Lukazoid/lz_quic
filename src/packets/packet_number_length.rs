use packets::packet_number::PacketNumber;
use primitives::u48::U48;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PacketNumberLength {
    OneByte,
    TwoBytes,
    FourBytes,
    SixBytes,
}

impl PacketNumberLength {
    /// Gets the length of the packet number in bytes.
    pub fn len(self) -> u8 {
        match self {
            PacketNumberLength::OneByte => 1,
            PacketNumberLength::TwoBytes => 2,
            PacketNumberLength::FourBytes => 4,
            PacketNumberLength::SixBytes => 6,
        }
    }

    pub fn bit_len(self) -> u8 {
        self.len() * 8
    }

    pub fn max_packet_number(self) -> PacketNumber {
        let maximum_packet_number: u64 = match self {
            PacketNumberLength::OneByte => u8::max_value().into(),
            PacketNumberLength::TwoBytes => u16::max_value().into(),
            PacketNumberLength::FourBytes => u32::max_value().into(),
            PacketNumberLength::SixBytes => U48::max_value().into(),
        };

        maximum_packet_number.into()
    }

    pub fn max_transmittable(self) -> PacketNumber {
        let bit_len = self.bit_len();
        2u64.pow((bit_len - 2) as u32).into()
    }
}
