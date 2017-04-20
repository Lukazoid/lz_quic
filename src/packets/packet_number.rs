#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PacketNumber(u64);

impl PacketNumber {
    /// Attempts to get the next `PacketNumber`.
    // `Option::None` is returned if the next packet number would exceed 64 bits in size.
    pub fn next(self) -> Option<PacketNumber> {
        self.0.checked_add(1).map(PacketNumber)
    }

    pub fn max_value() -> PacketNumber {
        PacketNumber(u64::max_value())
    }

    // Returns the "epochs" around this `PacketNumber` given the specified number of trailing bits are removed.
    pub fn epochs(self, remove_trailing_bits: u8) -> [PacketNumber; 3] {
        let delta = 1 << remove_trailing_bits;

        let epoch = self.0 & !(delta - 1);

        [(epoch - delta).into(), epoch.into(), (epoch + delta).into()]
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

#[cfg(test)]
mod tests {
    use super::*;
    use packets::partial_packet_number::PartialPacketNumber;

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

