use errors::*;
use primitives::{U48, AbsDelta};
use std::ops::Add;
use protocol::{Readable, Writable};
use std::io::{Read, Write};
use conv::{ConvAsUtil, UnwrapOk, Wrapping};
use smallvec::SmallVec;
use lz_diet::AdjacentBound;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PacketNumber(u64);

impl AdjacentBound for PacketNumber {
    fn is_immediately_before(&self, other: &Self) -> bool{
        (self.0 + 1) == other.0
    }

    fn is_immediately_after(&self, other: &Self) -> bool{
        self.0 == (other.0 + 1)
    }

    fn increment(&self) -> Self {
        (self.0 + 1).into()
    }

    fn decrement(&self) -> Self{
        (self.0 - 1).into()
    }

    fn increment_ref(&mut self){
        self.0 += 1;
    }

    fn decrement_ref(&mut self){
        self.0 -= 1;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PartialPacketNumberLength {
    OneByte,
    TwoBytes,
    FourBytes,
    SixBytes,
}

/// This represents a partial packet number consisting of only the lower bytes.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PartialPacketNumber {
    OneByte(u8),
    TwoBytes(u16),
    FourBytes(u32),
    SixBytes(U48),
}

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
    fn epochs(self, remove_trailing_bits: u8) -> SmallVec<[PacketNumber; 3]> {
        trace!("calculating epochs of packet {:?} after removal of {} trailing bits", self, remove_trailing_bits);

        let delta = 1 << remove_trailing_bits;

        trace!("packet number {:?} has a delta of {} after removal of {} trailing bits", self, delta, remove_trailing_bits);

        let epoch = self.0 & !(delta - 1);

        trace!("packet number {:?} has an epoch of {} after removal of {} trailing bits", self, epoch, remove_trailing_bits);

        let mut result = SmallVec::new();

        if let Some(first) = epoch.checked_sub(delta) {
            result.push(first.into());
        }

        result.push(epoch.into());

        if let Some(last) = epoch.checked_add(delta) {
            result.push(last.into())
        }

        debug!("calculated epochs {:?} of packet {:?} after removal of {} trailing bits", result, self, remove_trailing_bits);

        result
    }
}

impl Add<u64> for PacketNumber {
    type Output = PacketNumber;

    fn add(self, rhs: u64) -> Self::Output {
        let packet_number_value = u64::from(self) + rhs;

        PacketNumber::from(packet_number_value)
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

impl From<u64> for PacketNumber {
    fn from(value: u64) -> Self {
        PacketNumber(value)
    }
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
        } else if diff < PartialPacketNumberLength::SixBytes.threshold() {
            PartialPacketNumber::SixBytes(packet_number.0.approx_by::<Wrapping>().unwrap_ok())
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

        let packet_number = if let Some(largest_acknowledged) = largest_acknowledged {
            if let Some(next) = largest_acknowledged.next() {
                let epochs = largest_acknowledged.epochs(self.len().bit_len());

                let possible_packet_numbers = epochs.into_iter().map(|epoch| epoch + self);

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
            PacketNumber::from(u64::from(self))
        };

        debug!("infered packet number {:?} from partial packet number {:?} with a largest acknowledged packet number of {:?}", packet_number, self, largest_acknowledged);

        Ok(packet_number)
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
            PartialPacketNumberLength::SixBytes => {
                let value = U48::read(reader).chain_err(||ErrorKind::FailedToReadPartialPacketNumber)?;
                PartialPacketNumber::SixBytes(value)
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
            PartialPacketNumber::SixBytes(value) => value.write(writer),
        }.chain_err(||ErrorKind::FailedToWritePartialPacketNumber)?;

        debug!("written partial packet number {:?}", self);

        Ok(())
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
    use smallvec::{Array, SmallVec};

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
        assert_eq!(epochs, SmallVec::<[PacketNumber; 3]>::from_slice(&[PacketNumber::from(5436160), PacketNumber::from(5436416), PacketNumber::from(5436672)]));
    }

    #[test]
    fn epochs_returns_correct_epochs_2() {
        // Arrange
        let packet_number = PacketNumber::from(5436534);

        // Act
        let epochs = packet_number.epochs(16);

        // Assert
        assert_eq!(epochs, SmallVec::<[PacketNumber; 3]>::from_slice(&[PacketNumber::from(5308416), PacketNumber::from(5373952), PacketNumber::from(5439488)]));
    }

    #[test]
    fn epochs_returns_correct_epochs_3() {
        // Arrange
        let packet_number = PacketNumber::from(5436534);

        // Act
        let epochs = packet_number.epochs(1);

        // Assert
        assert_eq!(epochs, SmallVec::<[PacketNumber; 3]>::from_slice(&[PacketNumber::from(5436532), PacketNumber::from(5436534), PacketNumber::from(5436536)]));
    }

}

