use errors::*;
use std::convert::TryFrom;
use std::ops::Sub;

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

impl Sub for U24 {
    type Output = U24;

    fn sub(self, rhs: U24) -> Self::Output {
        U24(self.0 - rhs.0)
    }
}

impl From<u16> for U24 {
    fn from(value: u16) -> U24 {
        U24(value as u32)
    }
}

impl From<U24> for u32 {
    fn from(value: U24) -> u32 {
        value.0
    }
}

impl TryFrom<u32> for U24 {
    type Error = Error;

    fn try_from(value: u32) -> Result<U24> {
        if value > MAX.into() {
            bail!(ErrorKind::U32ToU24IntegerOverflow(value))
        } else {
            Ok(U24(value))
        }
    }
}

