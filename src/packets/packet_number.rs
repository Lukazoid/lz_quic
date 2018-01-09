use errors::*;
use primitives::AbsDelta;
use std::ops::Add;
use protocol::{Readable, Writable};
use std::io::{Read, Write};
use conv::{Wrapping, TryFrom};
use smallvec::SmallVec;
use rand::Rng;
use lz_diet::AdjacentBound;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PacketNumber(u64);

impl TryFrom<u64> for PacketNumber {
    type Err = Error;

    fn try_from(value: u64) -> Result<PacketNumber> {
        if value > PacketNumber::MAX.0 {
            bail!(ErrorKind::ValueExceedsTheMaximumPacketNumberValue);
        }
        Ok(PacketNumber(value))
    }
}

impl AdjacentBound for PacketNumber {
    fn is_immediately_before(&self, other: &Self) -> bool{
        (self.0 + 1) == other.0
    }

    fn is_immediately_after(&self, other: &Self) -> bool{
        self.0 == (other.0 + 1)
    }

    fn increment(&self) -> Self {
        let incremented = self.0 + 1;

        PacketNumber::try_from(incremented)
            .unwrap_or(PacketNumber(0))
    }

    fn decrement(&self) -> Self{
        let decremented = self.0 - 1;
        
        PacketNumber::try_from(decremented)
            .unwrap_or(PacketNumber::MAX)
    }

    fn increment_ref(&mut self){
        *self = self.increment();
    }

    fn decrement_ref(&mut self){
        *self = self.decrement();
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PartialPacketNumberLength {
    OneByte,
    TwoBytes,
    FourBytes
}

/// This represents a partial packet number consisting of only the lower bytes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PartialPacketNumber {
    OneByte(u8),
    TwoBytes(u16),
    FourBytes(u32)
}

impl PacketNumber {
    pub const MAX: PacketNumber = PacketNumber(4611686018427387903);

    /// Attempts to get the next `PacketNumber`.
    /// `Option::None` is returned if the next packet number would exceed `PacketNumber::max_value()`.
    pub fn next(self) -> Option<PacketNumber> {
        let incremented = self.0 + 1;
        
        PacketNumber::try_from(incremented)
            .ok()
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
        trace!("calculating epochs of packet {:?} after removal of {} trailing bits", self, remove_trailing_bits);

        let delta = 1 << remove_trailing_bits;

        trace!("packet number {:?} has a delta of {} after removal of {} trailing bits", self, delta, remove_trailing_bits);

        let epoch = self.0 & !(delta - 1);

        trace!("packet number {:?} has an epoch of {} after removal of {} trailing bits", self, epoch, remove_trailing_bits);

        let mut result = SmallVec::new();

        if let Some(first) = epoch.checked_sub(delta) {
            result.push(PacketNumber(first));
        }

        result.push(PacketNumber(epoch));

        if let Some(last) = epoch.checked_add(delta) {
            result.push(PacketNumber(last))
        }

        debug!("calculated epochs {:?} of packet {:?} after removal of {} trailing bits", result, self, remove_trailing_bits);

        result
    }
}

impl Readable for PacketNumber {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        trace!("reading packet number");

        let inner = u64::read(reader)?;
        if inner > PacketNumber::MAX.0 {
            bail!(ErrorKind::ValueExceedsTheMaximumPacketNumberValue);
        }

        let packet_number = PacketNumber(inner);

        debug!("read packet number {:?}", packet_number);
        Ok(packet_number)
    }
}

impl Writable for PacketNumber {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing packet number {:?}", self);

        self.0.write(writer)?;

        debug!("written packet number {:?}", self);
        Ok(())
    }
}

impl From<PacketNumber> for u64 {
    fn from(value: PacketNumber) -> Self {
        value.0
    }
}

impl PartialPacketNumberLength {
    /// Gets the length of the packet number in bytes.
    pub fn len(self) -> u8 {
        match self {
            PartialPacketNumberLength::OneByte => 1,
            PartialPacketNumberLength::TwoBytes => 2,
            PartialPacketNumberLength::FourBytes => 4,
        }
    }

    pub fn bit_len(self) -> u8 {
        self.len() * 8
    }

    fn threshold(self) -> u64 {
        2 << (self.bit_len() - 2)
    }
}

impl PartialPacketNumber {
    pub fn from_packet_number(packet_number: PacketNumber, lowest_unacknowledged: PacketNumber) -> Result<PartialPacketNumber> {
        trace!("calculating partial packet number for packet number {:?} with a lowest acknowledged packet number of {:?}", packet_number, lowest_unacknowledged);

        let diff = packet_number.0.checked_sub(lowest_unacknowledged.0)
            .ok_or_else(|| Error::from_kind(ErrorKind::FailedToBuildPartialPacketNumber))?;

        let partial_packet_number = if diff < PartialPacketNumberLength::OneByte.threshold() {
            PartialPacketNumber::OneByte(packet_number.0 as u8)
        } else if diff < PartialPacketNumberLength::TwoBytes.threshold() {
            PartialPacketNumber::TwoBytes(packet_number.0 as u16)
        } else if diff < PartialPacketNumberLength::FourBytes.threshold() {
            PartialPacketNumber::FourBytes(packet_number.0 as u32)
        } else {
            bail!(ErrorKind::FailedToBuildPartialPacketNumber)
        };

        debug!("calculated partial packet number {:?} for packet number {:?} with a lowest acknowledged packet number of {:?}", partial_packet_number, packet_number, lowest_unacknowledged);

        Ok(partial_packet_number)
    }

    pub fn infer_packet_number(self,
                               largest_acknowledged: Option<PacketNumber>)
                               -> Result<PacketNumber> {
        trace!("infering packet number from partial packet number {:?} with a largest acknowledged packet number of {:?}", self, largest_acknowledged);

        let self_as_integer = u32::from(self) as u64;

        let packet_number = if let Some(largest_acknowledged) = largest_acknowledged {
            if let Some(next) = largest_acknowledged.next() {
                let epochs = largest_acknowledged.epochs(self.len().bit_len());

                let possible_packet_numbers = epochs.into_iter().flat_map(|epoch| PacketNumber::try_from(epoch.0 + self_as_integer).ok());

                let next_u64: u64 = next.into();

                let closest =
                    possible_packet_numbers
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
        match self {
            PartialPacketNumber::OneByte(_) => PartialPacketNumberLength::OneByte,
            PartialPacketNumber::TwoBytes(_) => PartialPacketNumberLength::TwoBytes,
            PartialPacketNumber::FourBytes(_) => PartialPacketNumberLength::FourBytes,
        }
    }

    pub fn read<R: Read>(reader: &mut R, length: PartialPacketNumberLength) -> Result<Self> {
        trace!("reading partial packet number of length {:?}", length);

        let partial_packet_number = match length {
            PartialPacketNumberLength::OneByte => {
                let value = u8::read(reader).chain_err(||ErrorKind::FailedToReadPartialPacketNumber)?;
                PartialPacketNumber::OneByte(value)
            }
            PartialPacketNumberLength::TwoBytes => {
                let value = u16::read(reader).chain_err(||ErrorKind::FailedToReadPartialPacketNumber)?;
                PartialPacketNumber::TwoBytes(value)
            }
            PartialPacketNumberLength::FourBytes => {
                let value = u32::read(reader).chain_err(||ErrorKind::FailedToReadPartialPacketNumber)?;
                PartialPacketNumber::FourBytes(value)
            }
        };

        debug!("read partial packet number {:?}", partial_packet_number);

        Ok(partial_packet_number)
    }
}

impl Writable for PartialPacketNumber {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing partial packet number {:?}", self);

        match *self {
            PartialPacketNumber::OneByte(value) => value.write(writer),
            PartialPacketNumber::TwoBytes(value) => value.write(writer),
            PartialPacketNumber::FourBytes(value) => value.write(writer),
        }.chain_err(||ErrorKind::FailedToWritePartialPacketNumber)?;

        debug!("written partial packet number {:?}", self);

        Ok(())
    }
}

impl From<PartialPacketNumber> for u32 {
    fn from(value: PartialPacketNumber) -> u32 {
        match value {
            PartialPacketNumber::OneByte(value) => value as u32,
            PartialPacketNumber::TwoBytes(value) => value as u32,
            PartialPacketNumber::FourBytes(value) => value as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smallvec::{Array, SmallVec};

    #[test]
    fn partial_packet_number_from_small_packet_number_gets_inferred_correctly() {
        let lowest_unacknowledged = PacketNumber(1);
        for i in 1..10000 {
            let packet_number = PacketNumber(i);
            let partial_packet_number = PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged).unwrap();

            let inferred_packet_number = partial_packet_number.infer_packet_number(Some(lowest_unacknowledged)).unwrap();

            assert_eq!(packet_number, inferred_packet_number);
        }
    }

    #[test]
    fn partial_packet_number_from_small_packet_number_with_increasing_acknowledged_gets_inferred_correctly() {
        for i in 1..10000 {
            let packet_number = PacketNumber(i);
            let lowest_unacknowledged = PacketNumber(i/2);
            let partial_packet_number = PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged).unwrap();

            let inferred_packet_number = partial_packet_number.infer_packet_number(Some(lowest_unacknowledged)).unwrap();

            assert_eq!(packet_number, inferred_packet_number);
        }
    }

    #[test]
    fn partial_packet_number_from_packet_number_calculates_correctly_1(){
        let lowest_unacknowledged = PacketNumber(0x6afa2f);
        let packet_number = PacketNumber(0x6b4264);

        let partial_packet_number = PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged).unwrap();

        assert_matches!(partial_packet_number, PartialPacketNumber::TwoBytes(0x4264));
    }

    #[test]
    fn partial_packet_number_from_packet_number_calculates_correctly_2(){
        let lowest_unacknowledged = PacketNumber(0x6afa2f);
        let packet_number = PacketNumber(0x6bc107);

        let partial_packet_number = PartialPacketNumber::from_packet_number(packet_number, lowest_unacknowledged).unwrap();

        assert_matches!(partial_packet_number, PartialPacketNumber::FourBytes(0x6bc107));
    }

    #[test]
    fn infer_of_first_packet_returns_correct_packet_number() {
        // Act
        let packet_number = PartialPacketNumber::OneByte(1)
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
        let packet_number = PartialPacketNumber::OneByte(234)
            .infer_packet_number(largest_acknowledged)
            .unwrap();

        // Assert
        assert_eq!(packet_number, PacketNumber(5436650));
    }

    #[test]
    fn infer_of_two_byte_partial_packet_number_returns_correct_packet_number() {
        // Arrange
        let largest_acknowledged = Some(PacketNumber(0xaa82f30e));

        // Act
        let packet_number = PartialPacketNumber::TwoBytes(0x1f94)
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
        assert_eq!(epochs, SmallVec::<[PacketNumber; 3]>::from_slice(&[PacketNumber(5436160), PacketNumber(5436416), PacketNumber(5436672)]));
    }

    #[test]
    fn epochs_returns_correct_epochs_2() {
        // Arrange
        let packet_number = PacketNumber(5436534);

        // Act
        let epochs = packet_number.epochs(16);

        // Assert
        assert_eq!(epochs, SmallVec::<[PacketNumber; 3]>::from_slice(&[PacketNumber(5308416), PacketNumber(5373952), PacketNumber(5439488)]));
    }

    #[test]
    fn epochs_returns_correct_epochs_3() {
        // Arrange
        let packet_number = PacketNumber(5436534);

        // Act
        let epochs = packet_number.epochs(1);

        // Assert
        assert_eq!(epochs, SmallVec::<[PacketNumber; 3]>::from_slice(&[PacketNumber(5436532), PacketNumber(5436534), PacketNumber(5436536)]));
    }

}

