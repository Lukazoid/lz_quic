use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Sub;
use conv::{ApproxFrom, ValueInto, ValueFrom, DefaultApprox, Wrapping, NoError, PosOverflow};
use conv::misc::Saturated;

pub const MAX: U48 = U48(0xff_ff_ff_ff_ff_ff);
pub const MIN: U48 = U48(0);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct U48(u64);

impl Saturated for U48 {
    fn saturated_max() -> Self{
        MAX
    }
    fn saturated_min() -> Self{
        MIN
    }
}

macro_rules! widening_conv {
    ($($src:ty) *) => {
        $(
            impl TryFrom<U48> for $src {
                type Error = PosOverflow<U48>;

                fn try_from(value: U48) -> Result<Self, Self::Error> {
                    value.value_into()
                }
            }
            
            impl From<$src> for U48 {
                fn from(value: $src) -> Self {
                    U48(value as u64)
                }
            }

            impl ApproxFrom<$src, Wrapping> for U48 {
                type Err = NoError;

                fn approx_from(src: $src) -> Result<Self, Self::Err> {
                    let widened_src = src as u64;
                    let max_value = U48::max_value();
                    if widened_src > max_value.0 {
                        Ok(max_value)
                    } else {
                        Ok(U48(widened_src))
                    }
                }
            }

            impl ApproxFrom<$src, DefaultApprox> for U48 {
                type Err = NoError;

                fn approx_from(src: $src) -> Result<Self, Self::Err> {
                    src.value_into()
                }
            }

            impl ValueFrom<$src> for U48 {
                type Err = NoError;

                fn value_from(src: $src) -> Result<Self, Self::Err> {
                    Ok(U48(src as u64))
                }
            }

            impl ValueFrom<U48> for $src {
                type Err = PosOverflow<U48>;

                fn value_from(src: U48) -> Result<Self, Self::Err> {
                    if src.0 > <$src>::max_value() as u64 {
                        Err(PosOverflow(src))
                    } else {
                        Ok(src.0 as $src)
                    }
                }
            }
        )*
    };
}

macro_rules! narrowing_conv {
    ($($src:ty) *) => {
        $(
            impl TryFrom<$src> for U48 {
                type Error = PosOverflow<$src>;

                fn try_from(value: $src) -> Result<Self, Self::Error> {
                    value.value_into()
                }
            }

            impl From<U48> for $src {
                fn from(value: U48) -> Self {
                    value.0 as $src
                }
            }

            impl ApproxFrom<$src, Wrapping> for U48 {
                type Err = NoError;

                fn approx_from(src: $src) -> Result<Self, Self::Err> {
                    let max_value = U48::max_value();
                    if src > max_value.0 as $src {
                        Ok(max_value)
                    } else {
                        Ok(U48(src as u64))
                    }
                }
            }
            
            impl ApproxFrom<$src, DefaultApprox> for U48 {
                type Err = PosOverflow<$src>;

                fn approx_from(src: $src) -> Result<Self, Self::Err> {
                    src.value_into()
                }
            }

            impl ValueFrom<$src> for U48 {
                type Err = PosOverflow<$src>;

                fn value_from(src: $src) -> Result<Self, Self::Err> {
                    if src > U48::max_value().0 as $src {
                        Err(PosOverflow(src))
                    } else {
                        Ok(U48(src as u64))
                    }
                }
            }

            impl ValueFrom<U48> for $src {
                type Err = NoError;

                fn value_from(src: U48) -> Result<Self, Self::Err> {
                    Ok(src.0 as $src)
                }
            }
        )*
    };
}

widening_conv!(u8 u16 u32);

narrowing_conv!(u64);

impl Display for U48 {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {

        self.0.fmt(fmt)
    }
}

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
