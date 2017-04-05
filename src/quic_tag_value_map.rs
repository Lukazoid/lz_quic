use errors::*;
use quic_tag::QuicTag;
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use writable::Writable;
use readable::Readable;

#[derive(Debug, Clone, Default)]
pub struct QuicTagValueMap {
    entries: BTreeMap<QuicTag, Vec<u8>>,
}

struct IntermediateQuicTagValue {
    quic_tag: QuicTag,
    end_offset: u32,
}

impl Readable for IntermediateQuicTagValue {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        let quic_tag = QuicTag::read(reader)?;
        let end_offset = u32::read(reader)
            .chain_err(|| ErrorKind::UnableToReadQuicTagValueMap)?;

        Ok(IntermediateQuicTagValue {
            quic_tag: quic_tag,
            end_offset: end_offset,
        })
    }
}

impl QuicTagValueMap {
    pub fn read<R: Read>(reader: &mut R, count: usize) -> Result<Self> {

        // Read the QUIC tag and data offsets
        let mut intermediate_entries = Vec::with_capacity(count);

        for _ in 0..count {
            let intermediate_tag_value = IntermediateQuicTagValue::read(reader)?;
            intermediate_entries.push(intermediate_tag_value);
        }

        // Read the QUIC data for each tag
        let mut previous_end_offset = 0;
        let mut entries = BTreeMap::new();
        for intermediate_entry in intermediate_entries {
            let end_offset = intermediate_entry.end_offset;
            let length = previous_end_offset - end_offset;

            let mut data = Vec::with_capacity(length as usize);
            io::copy(&mut reader.take(length as u64), &mut data)
                .chain_err(|| ErrorKind::UnableToReadQuicTagValueMap)?;

            previous_end_offset = end_offset;

            entries.insert(intermediate_entry.quic_tag, data);
        }

        Ok(Self { entries: entries })
    }
}

impl Writable for QuicTagValueMap {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {

        let mut end_offset = 0;

        // Write the QUIC tags and the end data offsets
        for entry in &self.entries {
            entry.0
                .write(writer)
                .chain_err(|| ErrorKind::UnableToWriteQuicTagValueMap)?;

            let data = entry.1;
            end_offset += data.len() as u32;

            end_offset.write(writer)
                .chain_err(|| ErrorKind::UnableToWriteQuicTagValueMapEndOffset(end_offset))
                .chain_err(|| ErrorKind::UnableToWriteQuicTagValueMap)?;
        }

        // Write the actual data for each QUIC tag
        for entry in &self.entries {
            let data = entry.1;

            writer.write_all(data)
                .chain_err(|| ErrorKind::UnableToWriteQuicTagValue(*entry.0))
                .chain_err(|| ErrorKind::UnableToWriteQuicTagValueMap)?;
        }

        Ok(())
    }
}

impl QuicTagValueMap {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// There should be no need for this to be used publicly, use `set_value` with a `Writable` type instead.
    fn set_data<T: Into<Vec<u8>>>(&mut self, quic_tag: QuicTag, value: T) {
        self.entries.insert(quic_tag, value.into());
    }

    pub fn set_value<T: Writable + ?Sized>(&mut self, quic_tag: QuicTag, value: &T) {
        self.set_data(quic_tag, value.bytes());
    }

    /// There should be no need for this to be used publicly, use `get_optional_value`/`get_required_value` with a `Readable` type instead.
    fn get_data(&self, quic_tag: QuicTag) -> Option<&[u8]> {
        self.entries
            .get(&quic_tag)
            .map(AsRef::as_ref)
    }

    pub fn get_optional_value<T: Readable>(&self, quic_tag: QuicTag) -> Result<Option<T>> {
        if let Some(data) = self.get_data(quic_tag) {
            T::from_bytes(data)
                .map(|x| Some(x))
                .chain_err(|| ErrorKind::InvalidQuicTagValue(quic_tag))
        } else {
            Ok(None)
        }
    }

    pub fn get_required_value<T: Readable>(&self, quic_tag: QuicTag) -> Result<T> {
        if let Some(data) = self.get_data(quic_tag) {
            T::from_bytes(data).chain_err(|| ErrorKind::InvalidQuicTagValue(quic_tag))
        } else {
            bail!(ErrorKind::MissingQuicTag(quic_tag))
        }
    }

    pub fn get_optional_values<T: Readable>(&self, quic_tag: QuicTag) -> Result<Vec<T>> {
        if let Some(data) = self.get_data(quic_tag) {
            T::collection_from_bytes(data).chain_err(|| ErrorKind::InvalidQuicTagValue(quic_tag))
        } else {
            Ok(Vec::default())
        }
    }

    pub fn get_required_values<T: Readable>(&self, quic_tag: QuicTag) -> Result<Vec<T>> {
        if let Some(data) = self.get_data(quic_tag) {
            T::collection_from_bytes(data).chain_err(|| ErrorKind::InvalidQuicTagValue(quic_tag))
        } else {
            bail!(ErrorKind::MissingQuicTag(quic_tag))
        }
    }
}

#[cfg(test)]
mod tests {
    use errors::*;
    use super::*;
    use quic_tag::QuicTag;
    use quic_version::QuicVersion;

    #[test]
    fn get_optional_value_for_missing_returns_none() {
        // Arrange
        let quic_tag_value_map = QuicTagValueMap::default();

        // Act
        let result = quic_tag_value_map.get_optional_value::<QuicVersion>(QuicTag::Version);

        // Assert
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn get_optional_value_for_existent_returns_value() {
        // Arrange
        let mut quic_tag_value_map = QuicTagValueMap::default();
        quic_tag_value_map.set_value(QuicTag::Version, &QuicVersion::DRAFT_IETF_01);

        // Act
        let result = quic_tag_value_map.get_optional_value::<QuicVersion>(QuicTag::Version);

        // Assert
        assert!(matches!(result, Ok(Some(QuicVersion::DRAFT_IETF_01))));
    }

    #[test]
    fn get_required_value_for_missing_returns_error() {
        // Arrange
        let quic_tag_value_map = QuicTagValueMap::default();

        // Act
        let result = quic_tag_value_map.get_required_value::<QuicVersion>(QuicTag::Version);

        // Assert
        assert!(matches!(result, Err(Error(ErrorKind::MissingQuicTag(QuicTag::Version), _))));
    }
}