use std::convert::TryFrom;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::ops::Sub;
use conv::{ApproxFrom, ValueInto, ValueFrom, DefaultApprox, Wrapping, NoError, PosOverflow};
use conv::misc::Saturated;

pub const MAX: U24 = U24(0xff_ff_ff);
pub const MIN: U24 = U24(0);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct U24(u32);

impl Saturated for U24 {
    fn saturated_max() -> Self {
        MAX
    }
    fn saturated_min() -> Self {
        MIN
    }
}

macro_rules! widening_conv {
    ($($src:ty) *) => {
        $(
            impl TryFrom<U24> for $src {
                type Error = PosOverflow<U24>;

                fn try_from(value: U24) -> Result<Self, Self::Error> {
                    value.value_into()
                }
            }

            impl From<$src> for U24 {
                fn from(value: $src) -> Self {
                    U24(value as u32)
                }
            }
            
            impl ApproxFrom<$src, Wrapping> for U24 {
                type Err = NoError;

                fn approx_from(src: $src) -> Result<Self, Self::Err> {
                    let widened_src = src as u32;
                    let max_value = U24::max_value();
                    if widened_src > max_value.0 {
                        Ok(max_value)
                    } else {
                        Ok(U24(widened_src))
                    }
                }
            }

            impl ApproxFrom<$src, DefaultApprox> for U24 {
                type Err = NoError;

                fn approx_from(src: $src) -> Result<Self, Self::Err> {
                    U24::value_from(src)
                }
            }

            impl ValueFrom<$src> for U24 {
                type Err = NoError;

                fn value_from(src: $src) -> Result<Self, Self::Err> {
                    Ok(U24(src as u32))
                }
            }

            impl ValueFrom<U24> for $src {
                type Err = PosOverflow<U24>;

                fn value_from(src: U24) -> Result<Self, Self::Err> {
                    if src.0 > <$src>::max_value() as u32 {
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
            impl TryFrom<$src> for U24 {
                type Error = PosOverflow<$src>;

                fn try_from(value: $src) -> Result<Self, Self::Error> {
                    value.value_into()
                }
            }

            impl From<U24> for $src {
                fn from(value: U24) -> Self {
                    value.0 as $src
                }
            }

            impl ApproxFrom<$src, Wrapping> for U24 {
                type Err = NoError;

                fn approx_from(src: $src) -> Result<Self, Self::Err> {
                    let max_value = U24::max_value();
                    if src > max_value.0 as $src {
                        Ok(max_value)
                    } else {
                        Ok(U24(src as u32))
                    }
                }
            }
            
            impl ApproxFrom<$src, DefaultApprox> for U24 {
                type Err = PosOverflow<$src>;

                fn approx_from(src: $src) -> Result<Self, Self::Err> {
                    src.value_into()
                }
            }

            impl ValueFrom<$src> for U24 {
                type Err = PosOverflow<$src>;

                fn value_from(src: $src) -> Result<Self, Self::Err> {
                    if src > U24::max_value().0 as $src {
                        Err(PosOverflow(src))
                    } else {
                        Ok(U24(src as u32))
                    }
                }
            }

            impl ValueFrom<U24> for $src {
                type Err = NoError;

                fn value_from(src: U24) -> Result<Self, Self::Err> {
                    Ok(src.0 as $src)
                }
            }
        )*
    };
}

widening_conv!(u8 u16);

narrowing_conv!(u32 u64);

impl Display for U24 {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        self.0.fmt(fmt)
    }
}

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