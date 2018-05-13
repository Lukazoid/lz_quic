use conv::{TryFrom, TryInto};
use errors::*;
use lz_diet::AdjacentBound;
use primitives::AbsDelta;
use protocol::{Readable, Writable};
use rand::Rng;
use smallvec::SmallVec;
use std::io::{Read, Write};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PacketNumber(u64);

impl TryFrom<u64> for PacketNumber {
    type Err = Error;

    fn try_from(value: u64) -> Result<PacketNumber> {
        if value > PacketNumber::MAX.0 {
            bail!(ErrorKind::ValueExceedsTheMaximumPacketNumberValue(value));
        }
        Ok(PacketNumber(value))
    }
}

impl From<u32> for PacketNumber {
    fn from(value: u32) -> PacketNumber {
        PacketNumber(value as u64)
    }
}

impl From<u16> for PacketNumber {
    fn from(value: u16) -> PacketNumber {
        PacketNumber(value as u64)
    }
}

impl From<u8> for PacketNumber {
    fn from(value: u8) -> PacketNumber {
        PacketNumber(value as u64)
    }
}

impl AdjacentBound for PacketNumber {
    fn is_immediately_before(&self, other: &Self) -> bool {
        self.increment() == *other
    }

    fn is_immediately_after(&self, other: &Self) -> bool {
        other.is_immediately_before(self)
    }

    fn increment(&self) -> Self {
        let incremented = self.0 + 1;

        PacketNumber::try_from(incremented).unwrap_or(PacketNumber(0))
    }

    fn decrement(&self) -> Self {
        let decremented = self.0 - 1;

        PacketNumber::try_from(decremented).unwrap_or(PacketNumber::MAX)
    }

    fn increment_ref(&mut self) {
        *self = self.increment();
    }

    fn decrement_ref(&mut self) {
        *self = self.decrement();
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PartialPacketNumberLength {
    OneByte,
    TwoBytes,
    FourBytes,
}

/// This represents a partial packet number consisting of only the lower bytes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PartialPacketNumber(u32);

impl PacketNumber {
    pub const MAX: PacketNumber = PacketNumber(4611686018427387903);

    /// Attempts to get the next `PacketNumber`.
    /// `Option::None` is returned if the next packet number would exceed `PacketNumber::max_value()`.
    pub fn next(self) -> Option<PacketNumber> {
        let incremented = self.0 + 1;

        PacketNumber::try_from(incremented).ok()
    }

    pub fn max_value() -> PacketNumber {
        Self::MAX
    }

    pub fn generate<R: Rng>(rng: &mut R) -> PacketNumber {
        trace!("generating new packet number");

        // The initial value for packet number MUST be selected randomly from a
        // range between 0 and 2^32 - 1025 (inclusive)
        let inner = rng.gen_range(0u32, 4294966272u32);
        let packet_number = PacketNumber(inner as u64);
        debug!("generated new packet number {:?}", packet_number);

        packet_number
    }

    /// Returns the "epochs" around this `PacketNumber` given the specified number of trailing bits are removed.
    fn epochs(self, remove_trailing_bits: u8) -> SmallVec<[PacketNumber; 3]> {
        trace!(
            "calculating epochs of packet {:?} after removal of {} trailing bits",
            self,
            remove_trailing_bits
        );

        let delta = 1 << remove_trailing_bits;

        trace!(
            "packet number {:?} has a delta of {} after removal of {} trailing bits",
            self,
            delta,
            remove_trailing_bits
        );

        let epoch = self.0 & !(delta - 1);

        trace!(
            "packet number {:?} has an epoch of {} after removal of {} trailing bits",
            self,
            epoch,
            remove_trailing_bits
        );

        let mut result = SmallVec::new();

        if let Some(first) = epoch.checked_sub(delta) {
            result.push(PacketNumber(first));
        }

        result.push(PacketNumber(epoch));

        if let Some(last) = epoch.checked_add(delta) {
            result.push(PacketNumber(last))
        }

        debug!(
            "calculated epochs {:?} of packet {:?} after removal of {} trailing bits",
            result, self, remove_trailing_bits
        );

        result
    }
}

impl From<PacketNumber> for u64 {
    fn from(value: PacketNumber) -> Self {
        value.0
    }
}

impl PartialPacketNumberLength {
    pub fn available_bits_len(self) -> u8 {
        match self {
            PartialPacketNumberLength::OneByte => 7,
            PartialPacketNumberLength::TwoBytes => 14,
            PartialPacketNumberLength::FourBytes => 30,
        }
    }

    pub fn encoded_bits_len(self) -> u8 {
        match self {
            PartialPacketNumberLength::OneByte => 8,
            PartialPacketNumberLength::TwoBytes => 16,
            PartialPacketNumberLength::FourBytes => 32,
        }
    }

    fn threshold(self) -> u64 {
        (2 << (self.available_bits_len() - 1)) - 1
    }
}

impl PartialPacketNumber {
    pub const MAX: PartialPacketNumber = PartialPacketNumber(0x3FFFFFFF);

    pub fn from_packet_number(
        packet_number: PacketNumber,
        lowest_unacknowledged: PacketNumber,
    ) -> Result<PartialPacketNumber> {
        trace!("calculating partial packet number for packet number {:?} with a lowest acknowledged packet number of {:?}", packet_number, lowest_unacknowledged);

        let diff = packet_number
            .0
            .checked_sub(lowest_unacknowledged.0)
            .ok_or_else(|| Error::from_kind(ErrorKind::FailedToBuildPartialPacketNumber))?;

        let partial_packet_number = if diff <= PartialPacketNumberLength::OneByte.threshold() {
            (packet_number.0 as u8).into()
        } else if diff <= PartialPacketNumberLength::TwoBytes.threshold() {
            (packet_number.0 as u16).into()
        } else if diff <= PartialPacketNumberLength::FourBytes.threshold() {
            (packet_number.0 as u32)
                .try_into()
                .chain_err(|| ErrorKind::FailedToBuildPartialPacketNumber)?
        } else {
            bail!(ErrorKind::FailedToBuildPartialPacketNumber)
        };

        debug!("calculated partial packet number {:?} for packet number {:?} with a lowest acknowledged packet number of {:?}", partial_packet_number, packet_number, lowest_unacknowledged);

        Ok(partial_packet_number)
    }

    pub fn infer_packet_number(
        self,
        largest_acknowledged: Option<PacketNumber>,
    ) -> Result<PacketNumber> {
        trace!("infering packet number from partial packet number {:?} with a largest acknowledged packet number of {:?}", self, largest_acknowledged);

        let self_as_integer = u32::from(self) as u64;

        let packet_number = if let Some(largest_acknowledged) = largest_acknowledged {
            if let Some(next) = largest_acknowledged.next() {
                let epochs = largest_acknowledged.epochs(self.len().encoded_bits_len());

                let possible_packet_numbers = epochs
                    .into_iter()
                    .flat_map(|epoch| PacketNumber::try_from(epoch.0 + self_as_integer).ok());

                let next_u64: u64 = next.into();

                let closest = possible_packet_numbers
                    .min_by_key(|pn| u64::from(*pn).abs_delta(next_u64))
                    .expect("there should always be a closest as there are always 3 epochs");

                closest
            } else {
                bail!(ErrorKind::FailedToInferPacketNumber);
            }
        } else {
            PacketNumber(self_as_integer)
        };

        debug!("infered packet number {:?} from partial packet number {:?} with a largest acknowledged packet number of {:?}", packet_number, self, largest_acknowledged);

        Ok(packet_number)
    }

    pub fn len(self) -> PartialPacketNumberLength {
        let leading_zeros = self.0.leading_zeros();
        if leading_zeros >= 25 {
            PartialPacketNumberLength::OneByte
        } else if leading_zeros >= 18 {
            PartialPacketNumberLength::TwoBytes
        } else if leading_zeros >= 2 {
            PartialPacketNumberLength::FourBytes
        } else {
            unreachable!(
                "should be impossible to create a PartialPacketNumber of the value '{}'",
                self.0
            );
        }
    }
}

impl TryFrom<u32> for PartialPacketNumber {
    type Err = Error;

    fn try_from(value: u32) -> Result<PartialPacketNumber> {
        if value > PartialPacketNumber::MAX.0 {
            bail!(ErrorKind::ValueExceedsTheMaximumPartialPacketNumberValue(
                value
            ));
        }
        Ok(PartialPacketNumber(value))
    }
}

impl From<u16> for PartialPacketNumber {
    fn from(value: u16) -> PartialPacketNumber {
        PartialPacketNumber((value & 0x3FFF) as u32)
    }
}

impl From<u8> for PartialPacketNumber {
    fn from(value: u8) -> PartialPacketNumber {
        PartialPacketNumber(value as u32)
    }
}

impl Readable for PartialPacketNumber {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        trace!("reading partial packet number");

        let first_byte = u8::read(reader).chain_err(|| ErrorKind::FailedToReadPartialPacketNumber)?;

        let (first_byte, length) = match (first_byte >> 6) & 0b11 {
            0b11 => (first_byte & 0x3f, PartialPacketNumberLength::FourBytes),
            0b10 => (first_byte & 0x3f, PartialPacketNumberLength::TwoBytes),
            0b00 | 0b01 => (first_byte & 0x7f, PartialPacketNumberLength::OneByte),
            _ => unreachable!("there should only be 2 bits"),
        };

        let partial_packet_number = match length {
            PartialPacketNumberLength::OneByte => first_byte.into(),
            PartialPacketNumberLength::TwoBytes => {
                let second_byte =
                    u8::read(reader).chain_err(|| ErrorKind::FailedToReadPartialPacketNumber)?;
                let value = ((first_byte as u16) << 8) | second_byte as u16;
                value.into()
            }
            PartialPacketNumberLength::FourBytes => {
                let second_byte =
                    u8::read(reader).chain_err(|| ErrorKind::FailedToReadPartialPacketNumber)?;
                let last_2_bytes =
                    u16::read(reader).chain_err(|| ErrorKind::FailedToReadPartialPacketNumber)?;
                let value = ((first_byte as u32) << 24) | ((second_byte as u32) << 16)
                    | last_2_bytes as u32;
                value.try_into()?
            }
        };

        debug!("read partial packet number {:?}", partial_packet_number);

        Ok(partial_packet_number)
    }
}

impl Writable for PartialPacketNumber {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing partial packet number {:?}", self);

        match self.len() {
            PartialPacketNumberLength::OneByte => (0x7f & (self.0 as u8)).write(writer),
            PartialPacketNumberLength::TwoBytes => (0x8000 | (self.0 as u16)).write(writer),
            PartialPacketNumberLength::FourBytes => (0xC0000000 | self.0).write(writer),
        }.chain_err(|| ErrorKind::FailedToWritePartialPacketNumber)?;

        debug!("written partial packet number {:?}", self);

        Ok(())
    }
}

impl From<PartialPacketNumber> for u32 {
    fn from(value: PartialPacketNumber) -> u32 {
        value.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::{Array, SmallVec};

    #[test]
    fn partial_packet_number_length_one_byte_threshold() {
        assert_eq!(PartialPacketNumberLength::OneByte.threshold(), 0x7f);
    }

    #[test]
    fn partial_packet_number_length_two_bytes_threshold() {
        assert_eq!(PartialPacketNumberLength::TwoBytes.threshold(), 0x3fff);
    }

    #[test]
    fn partial_packet_number_length_four_bytes_threshold() {
        assert_eq!(PartialPacketNumberLength::FourBytes.threshold(), 0x3fffffff);
    }

    #[test]
    fn partial_packet_number_from_small_packet_number_gets_inferred_correctly() {
        let lowest_unacknowledged = PacketNumber(1);
        for i in 1..10000 {
            let packet_number = PacketNumber(i);
            let partial_packet_number =
                PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged)
                    .unwrap();

            let inferred_packet_number = partial_packet_number
                .infer_packet_number(Some(lowest_unacknowledged))
                .unwrap();

            assert_eq!(packet_number, inferred_packet_number);
        }
    }

    #[test]
    fn partial_packet_number_from_small_packet_number_with_increasing_acknowledged_gets_inferred_correctly(
) {
        for i in 1..10000 {
            let packet_number = PacketNumber(i);
            let lowest_unacknowledged = PacketNumber(i / 2);
            let partial_packet_number =
                PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged)
                    .unwrap();

            let inferred_packet_number = partial_packet_number
                .infer_packet_number(Some(lowest_unacknowledged))
                .unwrap();

            assert_eq!(packet_number, inferred_packet_number);
        }
    }

    #[test]
    fn partial_packet_number_from_packet_number_calculates_correctly_1() {
        let lowest_unacknowledged = PacketNumber(0x6afa2f);
        let packet_number = PacketNumber(0x6b4264);

        let partial_packet_number =
            PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged).unwrap();

        assert_eq!(
            partial_packet_number,
            PartialPacketNumber::try_from(0x6B4264).unwrap()
        );
    }

    #[test]
    fn partial_packet_number_from_packet_number_calculates_correctly_2() {
        let lowest_unacknowledged = PacketNumber(0x6BC102);
        let packet_number = PacketNumber(0x6bc107);

        let partial_packet_number =
            PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged).unwrap();

        assert_eq!(partial_packet_number, PartialPacketNumber::from(0x7u8));
    }

    #[test]
    fn infer_of_first_packet_returns_correct_packet_number() {
        // Act
        let packet_number = PartialPacketNumber::from(1u8)
            .infer_packet_number(None)
            .unwrap();

        // Assert
        assert_eq!(packet_number, PacketNumber(1));
    }

    #[test]
    fn infer_of_packet_returns_correct_packet_number() {
        // Arrange
        let largest_acknowledged = Some(PacketNumber(5436534));

        // Act
        let packet_number = PartialPacketNumber::from(234u8)
            .infer_packet_number(largest_acknowledged)
            .unwrap();

        // Assert
        assert_eq!(packet_number, PacketNumber(5439722));
    }

    #[test]
    fn infer_of_two_byte_partial_packet_number_returns_correct_packet_number() {
        // Arrange
        let largest_acknowledged = Some(PacketNumber(0xaa82f30e));

        // Act
        let packet_number = PartialPacketNumber::from(0x1f94u16)
            .infer_packet_number(largest_acknowledged)
            .unwrap();

        // Assert
        assert_eq!(packet_number, PacketNumber(0xaa831f94));
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
        let packet_number = PacketNumber(5436534);

        // Act
        let epochs = packet_number.epochs(8);

        // Assert
        assert_eq!(
            epochs,
            SmallVec::<[PacketNumber; 3]>::from_slice(&[
                PacketNumber(5436160),
                PacketNumber(5436416),
                PacketNumber(5436672)
            ])
        );
    }

    #[test]
    fn epochs_returns_correct_epochs_2() {
        // Arrange
        let packet_number = PacketNumber(5436534);

        // Act
        let epochs = packet_number.epochs(16);

        // Assert
        assert_eq!(
            epochs,
            SmallVec::<[PacketNumber; 3]>::from_slice(&[
                PacketNumber(5308416),
                PacketNumber(5373952),
                PacketNumber(5439488)
            ])
        );
    }

    #[test]
    fn epochs_returns_correct_epochs_3() {
        // Arrange
        let packet_number = PacketNumber(5436534);

        // Act
        let epochs = packet_number.epochs(1);

        // Assert
        assert_eq!(
            epochs,
            SmallVec::<[PacketNumber; 3]>::from_slice(&[
                PacketNumber(5436532),
                PacketNumber(5436534),
                PacketNumber(5436536)
            ])
        );
    }

}
