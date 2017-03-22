use std::cmp::Ordering;
use std::fmt::{Display, Debug, Error as FmtError, Formatter};
use std::num::FpCategory;
use conv::ApproxFrom;
use conv::errors::NoError;

const EXPONENT_BIT_COUNT: u16 = 5;
const MANTISSA_BIT_COUNT: u16 = 16 - EXPONENT_BIT_COUNT;
const EFFECTIVE_MANTISSA_BIT_COUNT: u16 = MANTISSA_BIT_COUNT + 1;

const EXPONENT_BIT_MASK: u16 = ((1 << EXPONENT_BIT_COUNT) - 1) << MANTISSA_BIT_COUNT;
const MANTISSA_BIT_MASK: u16 = (1 << MANTISSA_BIT_COUNT) - 1;

const MAX_INTEGER: u64 = 4396972769280;
pub const MAX: UF16 = UF16(0xffff);
pub const MIN: UF16 = UF16(0);

/// An unsigned 16-bit floating point number.
/// This vaguely represents the half-precision floating point format but with some changes for the Quic protocol.
/// These changes include:
///  - 11 explicit mantissa bits (instead of 10).
///  - No NaN representation.
///  - No Infinity representation.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct UF16(u16);

impl Debug for UF16 {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        let as_float = f32::from(*self);

        Debug::fmt(&as_float, fmt)
    }
}

impl Display for UF16 {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), FmtError> {
        let as_float = f32::from(*self);

        Display::fmt(&as_float, fmt)
    }
}

macro_rules! to_float {
    ($dst: ty) => {
        impl From<UF16> for $dst {
            fn from(value: UF16) -> $dst {
                (value.effective_mantissa() as $dst * (2.0 as $dst).powf(value.effective_exponent() as $dst))
            }
        }
    }
}

to_float!(f64);
to_float!(f32);

impl From<u8> for UF16 {
    fn from(value: u8) -> UF16 {
        UF16(value as u16)
    }
}

impl ApproxFrom<u8> for UF16 {
    type Err = NoError;

    fn approx_from(src: u8) -> Result<Self, Self::Err> {
        Ok(UF16::from(src))
    }
}

macro_rules! approx_from {
    ($src:ty) => {
        impl ApproxFrom<$src> for UF16 {
            type Err = NoError;

            fn approx_from(src: $src) -> Result<Self, Self::Err> {
                let result = if src < (1 as $src << EFFECTIVE_MANTISSA_BIT_COUNT) {
                    UF16(src as u16)
                } else if src >= (MAX_INTEGER as $src) {
                    MAX
                } else {
                    let mut value = src;
                    let mut exponent = 0u16;
                    let mut offset = 0b10000;
                    while offset > 0 {
                        if value >= (1 as $src << (MANTISSA_BIT_COUNT + offset)) {
                            exponent += offset;
                            value >>= offset;
                        }
                        offset >>= 1;
                    }

                    let inner = (value as u16) + (exponent << MANTISSA_BIT_COUNT);

                    UF16(inner)
                };

                Ok(result)
            }
        }
    };
}

approx_from!(u64);
approx_from!(u32);
approx_from!(u16);

impl UF16 {
    /// Creates a new `UF16` from its binary represenation.
    pub fn from_binary(binary: u16) -> UF16 {
        UF16(binary)
    }


    /// Gets the `UF16` in its binary represenation.
    pub fn to_binary(self) -> u16 {
        self.0
    }

    pub fn zero() -> UF16 {
        UF16(0)
    }

    pub fn one() -> UF16 {
        UF16(1)
    }

    pub fn min_value() -> UF16 {
        MIN
    }

    pub fn max_value() -> UF16 {
        MAX
    }

    fn explicit_exponent(self) -> u8 {
        let exponent_bits = (self.0 & EXPONENT_BIT_MASK) >> MANTISSA_BIT_COUNT;
        exponent_bits as u8
    }

    fn effective_exponent(self) -> u8 {
        let explicit_exponent = self.explicit_exponent();
        if explicit_exponent == 0 {
            0
        } else {
            explicit_exponent - 1
        }
    }

    fn explicit_mantissa(self) -> u16 {
        self.0 & MANTISSA_BIT_MASK
    }

    fn effective_mantissa(self) -> u16 {
        let explicit_mantissa = self.explicit_mantissa();
        if self.explicit_exponent() != 0 {
            // If there is an explicit exponent then we assume a high-order 12th bit
            0x800 | explicit_mantissa
        } else {
            explicit_mantissa
        }
    }

    pub fn is_nan(self) -> bool {
        self.classify() == FpCategory::Nan
    }

    pub fn is_infinite(self) -> bool {
        self.classify() == FpCategory::Infinite
    }

    pub fn is_finite(self) -> bool {
        match self.classify() {
            FpCategory::Infinite | FpCategory::Nan => false,
            _ => true,
        }
    }

    pub fn is_subnormal(self) -> bool {
        self.classify() == FpCategory::Subnormal
    }

    pub fn is_normal(self) -> bool {
        self.classify() == FpCategory::Normal
    }

    pub fn classify(self) -> FpCategory {
        if self.0 == 0 {
            FpCategory::Zero
        } else {
            FpCategory::Normal
        }
    }
}

impl PartialOrd for UF16 {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let self_as_float = f32::from(*self);
        let rhs_as_float = f32::from(*rhs);

        self_as_float.partial_cmp(&rhs_as_float)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use conv::ApproxFrom;

    #[test]
    pub fn max_is_correct_value() {
        assert_eq!(f32::from(MAX), 4396972769280f32);
    }

    macro_rules! approx_from_tests {
        ($($name:ident: $value:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    let (input, expected) : (u64, UF16) = $value;
                    
                    assert_eq!(expected, UF16::approx_from(input).unwrap());
                }
            )*
        }
    }

    approx_from_tests!{
        small_numbers_1: (0, UF16(0)),
        small_numbers_2: (1, UF16(1)),
        small_numbers_3: (2, UF16(2)),
        small_numbers_4: (3, UF16(3)),
        small_numbers_5: (4, UF16(4)),
        small_numbers_6: (5, UF16(5)),
        small_numbers_7: (6, UF16(6)),
        small_numbers_8: (7, UF16(7)),
        small_numbers_9: (15, UF16(15)),
        small_numbers_10: (31, UF16(31)),
        small_numbers_11: (42, UF16(42)),
        small_numbers_12: (123, UF16(123)),
        small_numbers_13: (1234, UF16(1234)),

        first_transition_1: (2046, UF16(2046)),
        first_transition_2: (2047, UF16(2047)),
        first_transition_3: (2048, UF16(2048)),
        first_transition_4: (2049, UF16(2049)),

        running_out_of_mantissa_1: (4094, UF16(4094)),
        running_out_of_mantissa_2: (4095, UF16(4095)),
        running_out_of_mantissa_3: (4096, UF16(4096)),
        running_out_of_mantissa_4: (4097, UF16(4096)),
        running_out_of_mantissa_5: (4098, UF16(4097)),
        running_out_of_mantissa_6: (4099, UF16(4097)),
        running_out_of_mantissa_7: (4100, UF16(4098)),
        running_out_of_mantissa_8: (4101, UF16(4098)),

        second_transition_1: (8190, UF16(6143)),
        second_transition_2: (8191, UF16(6143)),
        second_transition_3: (8192, UF16(6144)),
        second_transition_4: (8193, UF16(6144)),
        second_transition_5: (8194, UF16(6144)),
        second_transition_6: (8195, UF16(6144)),
        second_transition_7: (8196, UF16(6145)),
        second_transition_8: (8197, UF16(6145)),

        halfway_exponents_1: (0x7FF8000, UF16(0x87FF)),
        halfway_exponents_2: (0x7FFFFFF, UF16(0x87FF)),
        halfway_exponents_3: (0x8000000, UF16(0x8800)),
        halfway_exponents_4: (0xFFF0000, UF16(0x8FFF)),
        halfway_exponents_5: (0xFFFFFFF, UF16(0x8FFF)),
        halfway_exponents_6: (0x10000000, UF16(0x9000)),

        largest_exponent_1: (0x1FFFFFFFFFE, UF16(0xF7FF)),
        largest_exponent_2: (0x1FFFFFFFFFF, UF16(0xF7FF)),
        largest_exponent_3: (0x20000000000, UF16(0xF800)),
        largest_exponent_4: (0x20000000001, UF16(0xF800)),
        largest_exponent_5: (0x2003FFFFFFE, UF16(0xF800)),
        largest_exponent_6: (0x2003FFFFFFF, UF16(0xF800)),
        largest_exponent_7: (0x20040000000, UF16(0xF801)),
        largest_exponent_8: (0x20040000001, UF16(0xF801)),

        max_value_1: (0x3FF80000000, UF16(0xFFFE)),
        max_value_2: (0x3FFBFFFFFFF, UF16(0xFFFE)),
        max_value_3: (0x3FFC0000000, MAX),
        max_value_4: (0x3FFC0000001, MAX),
        max_value_5: (0x3FFFFFFFFFF, MAX),
        max_value_6: (0x40000000000, MAX),
        max_value_7: (0xFFFFFFFFFFFFFFFF, MAX),
    }

}