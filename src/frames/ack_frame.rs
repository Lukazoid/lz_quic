use conv::ValueInto;
use errors::*;
use protocol::{Readable, VarInt, Writable};
use std::io::{Read, Write};
use std::ops::Range;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct AckFrame {
    pub ack_delay: u64,
    pub ack_ranges_descending: Vec<Range<u64>>,
}

impl Readable for AckFrame {
    type Context = ();

    fn read_with_context<R: Read>(reader: &mut R, _: &Self::Context) -> Result<Self> {
        trace!("reading ack frame");

        let largest_acknowledged =
            VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadAckFrame)?;
        let ack_delay = VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadAckFrame)?;
        let ack_block_count = VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadAckFrame)?;
        let first_ack_block = VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadAckFrame)?;

        let largest_inclusive = largest_acknowledged.into_inner();
        let largest_exclusive = largest_inclusive + 1;
        let smallest = largest_inclusive
            .checked_sub(first_ack_block.into_inner())
            .ok_or_else(|| ErrorKind::FailedToReadAckFrame)?;

        let mut ack_ranges_descending = vec![(smallest..largest_exclusive)];

        let mut previous_smallest = smallest;

        for _ in 0..ack_block_count.into_inner() {
            let gap = VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadAckFrame)?;

            let largest_inclusive = previous_smallest
                .checked_sub(gap.into_inner())
                .ok_or_else(|| ErrorKind::FailedToReadAckFrame)?
                .checked_sub(2)
                .ok_or_else(|| ErrorKind::FailedToReadAckFrame)?;

            let largest_exclusive = largest_inclusive + 1;

            let ack_block = VarInt::read(reader).chain_err(|| ErrorKind::FailedToReadAckFrame)?;

            let smallest = largest_inclusive
                .checked_sub(ack_block.into_inner())
                .ok_or_else(|| ErrorKind::FailedToReadAckFrame)?;

            ack_ranges_descending.push((smallest..largest_exclusive));

            previous_smallest = smallest;
        }

        let ack_frame = Self {
            ack_delay: ack_delay.into(),
            ack_ranges_descending,
        };
        debug!("read ack frame {:?}", ack_frame);

        Ok(ack_frame)
    }
}

impl Writable for AckFrame {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing ack frame {:?}", self);

        let first_ack_range = self.ack_ranges_descending
            .get(0)
            .ok_or_else(|| ErrorKind::FailedToWriteAckFrame)?;

        let largest_inclusive = first_ack_range
            .end
            .checked_sub(1)
            .ok_or_else(|| ErrorKind::FailedToWriteAckFrame)?;

        let largest_acknowledged: VarInt = largest_inclusive
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;
        largest_acknowledged
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;

        let ack_delay: VarInt = self.ack_delay
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;
        ack_delay
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;

        let ack_block_count: VarInt = (self.ack_ranges_descending.len() - 1)
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;
        ack_block_count
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;

        let smallest = first_ack_range.start;

        let first_ack_block_range: VarInt = largest_inclusive
            .checked_sub(smallest)
            .ok_or_else(|| ErrorKind::FailedToWriteAckFrame)?
            .value_into()
            .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;
        first_ack_block_range
            .write(writer)
            .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;

        let mut previous_smallest = smallest;

        for ack_block in self.ack_ranges_descending.iter().skip(1) {
            let largest_exclusive = ack_block.end;

            let largest_inclusive = largest_exclusive
                .checked_sub(1)
                .ok_or_else(|| ErrorKind::FailedToWriteAckFrame)?;

            let gap: VarInt = previous_smallest
                .checked_sub(largest_inclusive)
                .ok_or_else(|| ErrorKind::FailedToWriteAckFrame)?
                .checked_sub(2)
                .ok_or_else(|| ErrorKind::FailedToWriteAckFrame)?
                .value_into()
                .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;

            gap.write(writer)
                .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;

            let smallest = ack_block.start;
            let ack_block_range: VarInt = largest_inclusive
                .checked_sub(smallest)
                .ok_or_else(|| ErrorKind::FailedToWriteAckFrame)?
                .value_into()
                .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;

            ack_block_range
                .write(writer)
                .chain_err(|| ErrorKind::FailedToWriteAckFrame)?;

            previous_smallest = smallest;
        }

        debug!("written ack frame {:?}", self);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::AckFrame;
    use protocol;

    #[test]
    fn write_read_ack_frame() {
        let ack_frame = AckFrame {
            ack_delay: 546,
            ack_ranges_descending: vec![(50..55), (46..49), (10..15)],
        };

        protocol::test_write_read(&ack_frame).unwrap();
    }
}
