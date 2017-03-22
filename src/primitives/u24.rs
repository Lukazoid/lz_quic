use errors::*;
use std::convert::TryFrom;

pub const MAX: U24 = U24(0xff_ff_ff);
pub const MIN: U24 = U24(0);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct U24(u32);

impl U24 {
    pub fn max_value() -> U24 {
        MAX
    }

    pub fn min_value() -> U24 {
        MIN
    }
}

impl From<U24> for u32 {
    fn from(value: U24) -> u32 {
        value.0
    }
}

impl TryFrom<u32> for U24 {
    type Err = Error;

    fn try_from(value: u32) -> Result<U24> {
        if value > MAX.into() {
            bail!(ErrorKind::U32ToU24IntegerOverflow(value))
        } else {
            Ok(U24(value))
        }
    }
}
