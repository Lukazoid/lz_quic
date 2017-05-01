use errors::*;
use primitives::u48::U48;
use primitives::abs_delta::AbsDelta;
use std::ops::Add;
use writable::Writable;
use readable::Readable;
use std::io::{Read, Write};
use conv::{ConvAsUtil, UnwrapOk, Wrapping};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PacketNumber(u64);

impl PacketNumber {
    /// Attempts to get the next `PacketNumber`.
    /// `Option::None` is returned if the next packet number would exceed `PacketNumber::max_value()`.
    pub fn next(self) -> Option<PacketNumber> {
        self.0.checked_add(1).map(PacketNumber)
    }

    pub fn max_value() -> PacketNumber {
        PacketNumber(u64::max_value())
    }

    /// Returns the "epochs" around this `PacketNumber` given the specified number of trailing bits are removed.
    fn epochs(self, remove_trailing_bits: u8) -> [PacketNumber; 3] {
        let delta = 1 << remove_trailing_bits;

        let epoch = self.0 & !(delta - 1);

        [(epoch.wrapping_sub(delta)).into(), epoch.into(), (epoch.wrapping_add(delta)).into()]
    }
}

impl From<PacketNumber> for u64 {
    fn from(value: PacketNumber) -> Self {
        value.0
    }
}

impl From<u64> for PacketNumber {
    fn from(value: u64) -> Self {
        PacketNumber(value)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PartialPacketNumberLength {
    OneByte,
    TwoBytes,
    FourBytes,
    SixBytes,
}

impl PartialPacketNumberLength {
    /// Gets the length of the packet number in bytes.
    pub fn len(self) -> u8 {
        match self {
            PartialPacketNumberLength::OneByte => 1,
            PartialPacketNumberLength::TwoBytes => 2,
            PartialPacketNumberLength::FourBytes => 4,
            PartialPacketNumberLength::SixBytes => 6,
        }
    }

    pub fn bit_len(self) -> u8 {
        self.len() * 8
    }

    fn threshold(self) -> u64 {
        2 << (self.bit_len() - 2)
    }
}

/// This represents a partial packet number consisting of only the lower bytes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PartialPacketNumber {
    OneByte(u8),
    TwoBytes(u16),
    FourBytes(u32),
    SixBytes(U48),
}

impl PartialPacketNumber {
    pub fn from_packet_number(packet_number: PacketNumber, lowest_unacknowledged: PacketNumber) -> Result<PartialPacketNumber> {
        let diff = packet_number.0.checked_sub(lowest_unacknowledged.0)
            .ok_or_else(|| Error::from_kind(ErrorKind::UnableToBuildPartialPacketNumber))?;

        let partial_packet_number = if diff < PartialPacketNumberLength::OneByte.threshold() {
            PartialPacketNumber::OneByte(packet_number.0 as u8)
        } else if diff < PartialPacketNumberLength::TwoBytes.threshold() {
            PartialPacketNumber::TwoBytes(packet_number.0 as u16)
        } else if diff < PartialPacketNumberLength::FourBytes.threshold() {
            PartialPacketNumber::FourBytes(packet_number.0 as u32)
        } else if diff < PartialPacketNumberLength::SixBytes.threshold() {
            PartialPacketNumber::SixBytes(packet_number.0.approx_by::<Wrapping>().unwrap_ok())
        } else {
            bail!(ErrorKind::UnableToBuildPartialPacketNumber)
        };

        Ok(partial_packet_number)
    }

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

    pub fn len(self) -> PartialPacketNumberLength {
        match self {
            PartialPacketNumber::OneByte(_) => PartialPacketNumberLength::OneByte,
            PartialPacketNumber::TwoBytes(_) => PartialPacketNumberLength::TwoBytes,
            PartialPacketNumber::FourBytes(_) => PartialPacketNumberLength::FourBytes,
            PartialPacketNumber::SixBytes(_) => PartialPacketNumberLength::SixBytes,
        }
    }

    pub fn read<R: Read>(reader: &mut R, length: PartialPacketNumberLength) -> Result<Self> {
        let partial_packet_number = match length {
            PartialPacketNumberLength::OneByte => {
                let value = u8::read(reader).chain_err(||ErrorKind::UnableToReadPartialPacketNumber)?;
                PartialPacketNumber::OneByte(value)
            }
            PartialPacketNumberLength::TwoBytes => {
                let value = u16::read(reader).chain_err(||ErrorKind::UnableToReadPartialPacketNumber)?;
                PartialPacketNumber::TwoBytes(value)
            }
            PartialPacketNumberLength::FourBytes => {
                let value = u32::read(reader).chain_err(||ErrorKind::UnableToReadPartialPacketNumber)?;
                PartialPacketNumber::FourBytes(value)
            }
            PartialPacketNumberLength::SixBytes => {
                let value = U48::read(reader).chain_err(||ErrorKind::UnableToReadPartialPacketNumber)?;
                PartialPacketNumber::SixBytes(value)
            }
        };

        Ok(partial_packet_number)
    }
}

impl Writable for PartialPacketNumber {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        match *self {
            PartialPacketNumber::OneByte(value) => value.write(writer),
            PartialPacketNumber::TwoBytes(value) => value.write(writer),
            PartialPacketNumber::FourBytes(value) => value.write(writer),
            PartialPacketNumber::SixBytes(value) => value.write(writer),
        }.chain_err(||ErrorKind::UnableToWritePartialPacketNumber)
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


impl Add<PartialPacketNumber> for PacketNumber {
    type Output = PacketNumber;

    fn add(self, rhs: PartialPacketNumber) -> Self::Output {
        let packet_number_value = u64::from(self) + u64::from(rhs);

        PacketNumber::from(packet_number_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_packet_number_from_small_packet_number_gets_inferred_correctly() {
        let lowest_unacknowledged = PacketNumber::from(1);
        for i in 1..10000 {
            let packet_number = PacketNumber::from(i);
            let partial_packet_number = PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged).unwrap();

            let inferred_packet_number = partial_packet_number.infer_packet_number(Some(lowest_unacknowledged)).unwrap();

            assert_eq!(packet_number, inferred_packet_number);
        }
    }

    #[test]
    fn partial_packet_number_from_small_packet_number_with_increasing_acknowledged_gets_inferred_correctly() {
        for i in 1..10000 {
            let packet_number = PacketNumber::from(i);
            let lowest_unacknowledged = PacketNumber::from(i/2);
            let partial_packet_number = PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged).unwrap();

            let inferred_packet_number = partial_packet_number.infer_packet_number(Some(lowest_unacknowledged)).unwrap();

            assert_eq!(packet_number, inferred_packet_number);
        }
    }

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

    #[test]
    fn next_returns_none_on_overflow() {
        // Arrange
        let packet_number = PacketNumber::max_value();

        // Act
        let next = packet_number.next();

        // Assert
        assert_eq!(next, None);
    }

    #[test]
    fn epochs_returns_correct_epochs() {
        // Arrange
        let packet_number = PacketNumber::from(5436534);

        // Act
        let epochs = packet_number.epochs(8);

        // Assert
        assert_eq!(epochs, [PacketNumber::from(5436160), PacketNumber::from(5436416), PacketNumber::from(5436672)]);
    }

    #[test]
    fn epochs_returns_correct_epochs_2() {
        // Arrange
        let packet_number = PacketNumber::from(5436534);

        // Act
        let epochs = packet_number.epochs(16);

        // Assert
        assert_eq!(epochs, [PacketNumber::from(5308416), PacketNumber::from(5373952), PacketNumber::from(5439488)]);
    }

    #[test]
    fn epochs_returns_correct_epochs_3() {
        // Arrange
        let packet_number = PacketNumber::from(5436534);

        // Act
        let epochs = packet_number.epochs(1);

        // Assert
        assert_eq!(epochs, [PacketNumber::from(5436532), PacketNumber::from(5436534), PacketNumber::from(5436536)]);
    }

}

