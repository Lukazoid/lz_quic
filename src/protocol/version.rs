use errors::*;
use std::io::{Read, Write};
use protocol::{Readable, Writable};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version(u32);

const IETF_DRAFT_MASK: u32 = 0xff000000;

impl Version {
    pub fn is_force_negotiation(self) -> bool {
        const FORCE_NEGOTATION_MASK: u32 = 0x0a0a0a0a;

        (self.0 & FORCE_NEGOTATION_MASK) == FORCE_NEGOTATION_MASK
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

pub const DRAFT_IETF_08: Version = Version(0xff000008);

pub static SUPPORTED_VERSIONS: &'static [Version] = &[DRAFT_IETF_08];

impl Version {
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
