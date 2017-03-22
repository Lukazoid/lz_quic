use errors::*;
use std::convert::TryFrom;

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

impl From<U48> for u64 {
    fn from(value: U48) -> u64 {
        value.0
    }
}

impl TryFrom<u64> for U48 {
    type Err = Error;

    fn try_from(value: u64) -> Result<U48> {
        if value > MAX.into() {
            bail!(ErrorKind::U64ToU48IntegerOverflow(value))
        } else {
            Ok(U48(value))
        }
    }
}