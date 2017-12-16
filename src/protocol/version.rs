use errors::*;
use std::io::{Read, Write};
use protocol::{Readable, Writable};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u32);

const IETF_DRAFT_MASK: u32 = 0xff000000;

static SUPPORTED_VERSIONS: &'static [Version] = &[Version::DRAFT_IETF_08];

impl Version {
    pub const NEGOTATION: Version = Version(0);

    pub const DRAFT_IETF_08: Version = Version(0xff000008);

    pub fn is_version_negotation(self) -> bool {
        self.0 == 0    
    }

    pub fn is_force_negotiation(self) -> bool {
        (self.0 & 0x0f0f0f0f) == 0x0a0a0a0a
    }

    pub fn is_ietf_consensus_reserved(self) -> bool {
        const IETF_CONSENSUS_MASK: u32 = 0x0000FFFF;

        (self.0 & IETF_CONSENSUS_MASK) == self.0
    }

    pub fn is_ietf_draft(self) -> bool {
        (self.0 & IETF_DRAFT_MASK) == IETF_DRAFT_MASK
    }

    pub fn ietf_draft_number(self) -> Option<u32> {
        if self.is_ietf_draft() {
            Some(self.0 - IETF_DRAFT_MASK)
        } else {
            None
        }
    }
    
    pub fn find_highest_supported(other: &HashSet<Version>) -> Option<Version> {
        trace!("finding highest supported version from {:?}", other);
        // Find the first supported version going from highest -> lowest
        let highest_version = SUPPORTED_VERSIONS
            .iter()
            .rev()
            .find(|v| other.contains(v))
            .map(|v| *v);

        if let Some(highest_version) = highest_version {
            debug!("found supported version {:?}", highest_version);
            Some(highest_version)
        } else {
            debug!("unable to find supported version");
            None
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.0.fmt(f)
    }
}

impl Writable for Version {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        trace!("writing version {:?}", self);
        self.0.write(writer)?;
        debug!("written version {:?}", self);

        Ok(())
    }
}

impl Readable for Version {
    fn read<R: Read>(reader: &mut R) -> Result<Version> {
        trace!("reading version");
        let version = u32::read(reader).map(Version)?;
        debug!("read version {:?}", version);
        Ok(version)
    }
}

#[cfg(test)]
mod tests {
    use super::Version;
    use std::collections::HashSet;

    #[test]
    pub fn is_version_negotation_returns_true_for_negotation() {
        assert!(Version::NEGOTATION.is_version_negotation());    
    }
    
    #[test]
    pub fn is_version_negotation_returns_false_for_other_version() {
        let version = Version(15);

        assert_eq!(version.is_version_negotation(), false);    
    }

    #[test]
    pub fn is_force_negotation_returns_true_for_force_negotation_version() {
        let version = Version(0x1a2a3a4a);

        assert!(version.is_force_negotiation());
    }

    #[test]
    pub fn is_force_negotation_returns_false_for_normal_version() {
        let version = Version(0x162a3a4a);

        assert_eq!(version.is_force_negotiation(), false);
    }

    #[test]
    pub fn ietf_draft_version_works() {
        let version = Version(0xff00000D);

        assert!(version.is_ietf_draft());
        assert_eq!(version.ietf_draft_number(), Some(13));
    }

    #[test]
    pub fn find_highest_supported_returns_none_for_unsupported() {
        let mut available = HashSet::new();
        available.insert(Version(0x00001234));

        let highest_supported = Version::find_highest_supported(&available);

        assert_eq!(highest_supported, None);
    }

    #[test]
    pub fn find_highest_supported_returns_version_for_supported() {
        let mut available = HashSet::new();
        available.insert(Version::DRAFT_IETF_08);

        let highest_supported = Version::find_highest_supported(&available);

        assert_eq!(highest_supported, Some(Version::DRAFT_IETF_08));
    }
}
