use errors::*;
use primitives::u48::U48;
use primitives::abs_delta::AbsDelta;
use packets::packet_number_length::PacketNumberLength;
use packets::packet_number::PacketNumber;
use std::ops::Add;

/// This represents a partial packet number consisting of only the lower bytes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PartialPacketNumber {
    OneByte(u8),
    TwoBytes(u16),
    FourBytes(u32),
    SixBytes(U48),
}

impl Add<PartialPacketNumber> for PacketNumber {
    type Output = PacketNumber;

    fn add(self, rhs: PartialPacketNumber) -> Self::Output {
        let packet_number_value = u64::from(self) + u64::from(rhs);

        PacketNumber::from(packet_number_value)
    }
}

impl PartialPacketNumber {
    pub fn infer_packet_number(self,
                               largest_acknowledged: Option<PacketNumber>)
                               -> Result<PacketNumber> {

        if let Some(largest_acknowledged) = largest_acknowledged {
            if let Some(next) = largest_acknowledged.next() {
                let epochs = largest_acknowledged.epochs(self.len().bit_len());

                let possible_packet_numbers = epochs.into_iter().map(|epoch| *epoch + self);

                let next_u64: u64 = next.into();

                let closest =
                    possible_packet_numbers
                        .min_by_key(|pn| u64::from(*pn).abs_delta(next_u64))
                        .expect("there should always be a closest as there are always 3 epochs");

                Ok(closest)
            } else {
                bail!(ErrorKind::UnableToInferPacketNumber);
            }
        } else {
            Ok(PacketNumber::from(u64::from(self)))
        }

    }

    pub fn len(self) -> PacketNumberLength {
        match self {
            PartialPacketNumber::OneByte(_) => PacketNumberLength::OneByte,
            PartialPacketNumber::TwoBytes(_) => PacketNumberLength::TwoBytes,
            PartialPacketNumber::FourBytes(_) => PacketNumberLength::FourBytes,
            PartialPacketNumber::SixBytes(_) => PacketNumberLength::SixBytes,
        }
    }
}

impl From<PartialPacketNumber> for u64 {
    fn from(value: PartialPacketNumber) -> u64 {
        match value {
            PartialPacketNumber::OneByte(value) => value as u64,
            PartialPacketNumber::TwoBytes(value) => value as u64,
            PartialPacketNumber::FourBytes(value) => value as u64,
            PartialPacketNumber::SixBytes(value) => value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use packets::packet_number::PacketNumber;

    #[test]
    fn infer_of_first_packet_returns_correct_packet_number() {
        // Act
        let packet_number = PartialPacketNumber::OneByte(1)
            .infer_packet_number(None)
            .unwrap();

        // Assert
        assert_eq!(packet_number, PacketNumber::from(1));
    }

    #[test]
    fn infer_of_packet_returns_correct_packet_number() {
        // Arrange
        let largest_acknowledged = Some(PacketNumber::from(5436534));

        // Act
        let packet_number = PartialPacketNumber::OneByte(234)
            .infer_packet_number(largest_acknowledged)
            .unwrap();

        // Assert
        assert_eq!(packet_number, PacketNumber::from(5436650));
    }
}