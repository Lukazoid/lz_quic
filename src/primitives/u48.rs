use errors::*;
use std::convert::TryFrom;
use std::ops::Sub;

pub const MAX: U48 = U48(0xff_ff_ff_ff_ff_ff);
pub const MIN: U48 = U48(0);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct U48(u64);

impl U48 {
    pub fn max_value() -> U48 {
        MAX
    }

    pub fn min_value() -> U48 {
        MIN
    }
}

impl Sub for U48 {
    type Output = U48;

    fn sub(self, rhs: U48) -> Self::Output {
        U48(self.0 - rhs.0)
    }
}

impl From<u32> for U48 {
    fn from(value: u32) -> U48 {
        U48(value as u64)
    }
}

impl From<U48> for u64 {
    fn from(value: U48) -> u64 {
        value.0
    }
}

impl TryFrom<u64> for U48 {
    type Error = Error;

    fn try_from(value: u64) -> Result<U48> {
        if value > MAX.into() {
            bail!(ErrorKind::U64ToU48IntegerOverflow(value))
        } else {
            Ok(U48(value))
        }
    }
}
