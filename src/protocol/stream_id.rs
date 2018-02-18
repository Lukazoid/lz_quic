use errors::*;
use rand::Rng;
use std::fmt::{Display, Formatter, Result as FmtResult};
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};
use conv::TryFrom;
use num::Integer;
use protocol::{Perspective, Readable, StreamType, VarInt, Writable};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct StreamId(u64);

bitflags!(
    flags StreamIdFlags: u64 {
        const CLIENT         = 0b01,
        const UNIDIRECTIONAL = 0b10,
    }
);

impl StreamId {
    pub fn generate<R: Rng>(
        rng: &mut R,
        perspective: Perspective,
        stream_type: StreamType,
    ) -> StreamId {
        trace!("generating new stream id");

        let mut flags = StreamIdFlags::empty();
        if matches!(perspective, Perspective::Client) {
            flags.insert(CLIENT);
        }

        if matches!(stream_type, StreamType::Unidirectional) {
            flags.insert(UNIDIRECTIONAL);
        }

        let inner = rng.next_u64() | flags.bits();

        let stream_id = StreamId(inner);
        debug!("generated stream id {:?}", stream_id);
        stream_id
    }

    pub fn first_server_stream_id() -> Self {
        StreamId(2)
    }

    pub fn first_client_stream_id() -> Self {
        StreamId(1)
    }

    pub fn crypto_stream_id() -> Self {
        StreamId(0)
    }

    pub fn is_crypto_stream(self) -> bool {
        self.0 == 0
    }

    pub fn initiator(self) -> Perspective {
        if self.0.is_even() {
            Perspective::Client
        } else {
            Perspective::Server
        }
    }

    pub fn is_server_initiated(self) -> bool {
        matches!(self.initiator(), Perspective::Server)
    }

    pub fn is_client_initiated(self) -> bool {
        matches!(self.initiator(), Perspective::Client)
    }

    pub fn stream_type(self) -> StreamType {
        if self.0 & 0x2 == 0 {
            StreamType::Bidirectional
        } else {
            StreamType::Unidirectional
        }
    }

    pub fn is_unidirectional(self) -> bool {
        matches!(self.stream_type(), StreamType::Unidirectional)
    }

    pub fn is_bidirectional(self) -> bool {
        matches!(self.stream_type(), StreamType::Bidirectional)
    }

    pub fn next(self) -> Self {
        StreamId(self.0 + 2)
    }
}

impl Writable for StreamId {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing stream id {:?}", self);

        let var_int = VarInt::<u64>::try_from(self.0)?;

        var_int.write(writer)?;

        debug!("written stream id {:?}", self);

        Ok(())
    }
}

impl Readable for StreamId {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        trace!("reading stream id");

        let var_int: VarInt<u64> = Readable::read(reader)?;

        let stream_id = StreamId(var_int.into_inner());

        debug!("read stream id {:?}", stream_id);

        Ok(stream_id)
    }
}

impl From<u64> for StreamId {
    fn from(value: u64) -> Self {
        StreamId(value)
    }
}

impl Display for StreamId {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}
