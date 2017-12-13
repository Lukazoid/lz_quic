use errors::*;
use handshake::Tag;
use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use protocol::{Readable, Writable};
use std::iter::FromIterator;
use debugit::DebugIt;

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct TagValueMap {
    entries: BTreeMap<Tag, Vec<u8>>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct IntermediateTagValue {
    tag: Tag,
    end_offset: u32,
}

impl Readable for IntermediateTagValue {
    fn read<R: Read>(reader: &mut R) -> Result<Self> {
        trace!("reading intermediate tag value");
        let tag = Tag::read(reader)?;
        let end_offset = u32::read(reader)
            .chain_err(|| ErrorKind::FailedToReadTagValueMap)?;

        let intermediate_tag_value = IntermediateTagValue {
            tag: tag,
            end_offset: end_offset,
        };
        debug!("read intermediate tag value {:?}", intermediate_tag_value);

        Ok(intermediate_tag_value)
    }
}

impl TagValueMap {
    pub fn read<R: Read>(reader: &mut R, count: usize) -> Result<Self> {
        trace!("reading tag value map");

        trace!("reading {} intermediate tag value entries", count);
        // Read the QUIC tag and data offsets
        let mut intermediate_entries = Vec::with_capacity(count);

        for _ in 0..count {
            let intermediate_tag_value = IntermediateTagValue::read(reader)?;
            intermediate_entries.push(intermediate_tag_value);
        }
        debug!("read {} intermediate tag value entries", count);

        trace!("reading tag value map data");
        // Read the QUIC data for each tag
        let mut previous_end_offset = 0;
        let mut entries = BTreeMap::new();
        for intermediate_entry in intermediate_entries {
            let end_offset = intermediate_entry.end_offset;
            let length = end_offset - previous_end_offset;
            trace!("reading {} bytes of tag value data from offset {}", length, end_offset);
            
            let mut data = Vec::with_capacity(length as usize);
            io::copy(&mut reader.take(length as u64), &mut data)
                .chain_err(|| ErrorKind::FailedToReadTagValueMap)?;

            debug!("read {} bytes of tag value data from offset {}", length, end_offset);
            previous_end_offset = end_offset;

            entries.insert(intermediate_entry.tag, data);
        }

        let tag_value_map = Self { entries: entries };
        debug!("read tag value map {:?}", tag_value_map);
        Ok(tag_value_map)
    }
}

impl Writable for TagValueMap {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing tag value map {:?}", self);

        trace!("writing tags and data offsets");
        let mut end_offset = 0;

        // Write the QUIC tags and the end data offsets
        for entry in &self.entries {
            trace!("writing tag {:?}", entry.0);
            entry.0
                .write(writer)
                .chain_err(|| ErrorKind::FailedToWriteTagValueMap)?;
            debug!("written tag {:?}", entry.0);

            let data = entry.1;
            end_offset += data.len() as u32;
            trace!("writing data offset {}", end_offset);
            end_offset.write(writer)
                .chain_err(|| ErrorKind::FailedToWriteTagValueMapEndOffset(end_offset))
                .chain_err(|| ErrorKind::FailedToWriteTagValueMap)?;
            debug!("written data offset {}", end_offset);
        }
        debug!("written tags and data offsets");

        trace!("writing data");
        // Write the actual data for each QUIC tag
        for entry in &self.entries {
            let data = entry.1;

            writer.write_all(data)
                .chain_err(|| ErrorKind::FailedToWriteTagValue(*entry.0))
                .chain_err(|| ErrorKind::FailedToWriteTagValueMap)?;
        }
        debug!("written data");

        debug!("written tag value map {:?}", self);

        Ok(())
    }
}

impl TagValueMap {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// There should be no need for this to be used publicly, use `set_value` with a `Writable` type instead.
    fn set_data<T: Into<Vec<u8>>>(&mut self, tag: Tag, value: T) {
        trace!("setting data for tag {:?} to value {:?}", tag, DebugIt(&value));
        self.entries.insert(tag, value.into());
        debug!("set data for tag {:?}", tag);
    }

    pub fn set_value<T: Writable + ?Sized>(&mut self, tag: Tag, value: &T) {
        self.set_data(tag, value.bytes());
    }

    /// There should be no need for this to be used publicly, use `get_optional_value`/`get_required_value` with a `Readable` type instead.
    fn get_data(&self, tag: Tag) -> Option<&[u8]> {
        trace!("getting data of tag {:?}", tag);
        if let Some(ref bytes) = self.entries.get(&tag) {
            debug!("found data {:?} for tag {:?}", bytes, tag);
            Some(&bytes[..])
        } else {
            debug!("no data found for tag {:?}", tag);
            None
        }
    }

    pub fn get_optional_value<T: Readable>(&self, tag: Tag) -> Result<Option<T>> {
        if let Some(data) = self.get_data(tag) {
            let value = T::from_bytes(data)
                .chain_err(|| ErrorKind::InvalidTagValue(tag))?;
            
            debug!("found value {:?} for tag {:?}", DebugIt(&value), tag);

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub fn get_required_value<T: Readable>(&self, tag: Tag) -> Result<T> {
        if let Some(data) = self.get_data(tag) {
            let value = T::from_bytes(data).chain_err(|| ErrorKind::InvalidTagValue(tag))?;

            debug!("found value {:?} for tag {:?}", DebugIt(&value), tag);

            Ok(value)
        } else {
            bail!(ErrorKind::MissingTag(tag))
        }
    }

    pub fn get_optional_values<C: Default + FromIterator<T>, T: Readable>(&self, tag: Tag) -> Result<C> {
        if let Some(data) = self.get_data(tag) {
            let values = T::collect_from_bytes(data).chain_err(|| ErrorKind::InvalidTagValue(tag))?;

            debug!("found values {:?} for tag {:?}", DebugIt(&values), tag);

            Ok(values)
        } else {
            Ok(C::default())
        }
    }

    pub fn get_required_values<C: FromIterator<T>, T: Readable>(&self, tag: Tag) -> Result<C> {
        if let Some(data) = self.get_data(tag) {
            let values = T::collect_from_bytes(data).chain_err(|| ErrorKind::InvalidTagValue(tag))?;

            debug!("found values {:?} for tag {:?}", DebugIt(&values), tag);
            
            Ok(values)
        } else {
            bail!(ErrorKind::MissingTag(tag))
        }
    }
}

#[cfg(test)]
mod tests {
    use errors::*;
    use super::*;
    use handshake::Tag;
    use protocol::{version, Version, Writable};
    use std::io::Cursor;

    #[test]
    fn get_optional_value_for_missing_returns_none() {
        // Arrange
        let tag_value_map = TagValueMap::default();

        // Act
        let result = tag_value_map.get_optional_value::<Version>(Tag::Version);

        // Assert
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn get_optional_value_for_existent_returns_value() {
        // Arrange
        let mut tag_value_map = TagValueMap::default();
        tag_value_map.set_value(Tag::Version, &version::DRAFT_IETF_08);

        // Act
        let result = tag_value_map.get_optional_value::<Version>(Tag::Version);

        // Assert
        assert!(matches!(result, Ok(Some(version::DRAFT_IETF_08))));
    }

    #[test]
    fn get_required_value_for_missing_returns_error() {
        // Arrange
        let tag_value_map = TagValueMap::default();

        // Act
        let result = tag_value_map.get_required_value::<Version>(Tag::Version);

        // Assert
        assert!(matches!(result, Err(Error(ErrorKind::MissingTag(Tag::Version), _))));
    }

    #[test]
    fn serialization_works() {
        let mut vec = Vec::new();

        let mut tag_value_map = TagValueMap::default();
        tag_value_map.set_value(Tag::Version, &version::DRAFT_IETF_08);

        tag_value_map.write_to_vec(&mut vec);

        let read: TagValueMap = TagValueMap::read(&mut Cursor::new(vec), 1).unwrap();

        assert_eq!(tag_value_map, read);
    }
}