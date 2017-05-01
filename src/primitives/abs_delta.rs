use primitives::u24::U24;
use primitives::u48::U48;

/// A trait for getting the absolute delta between two values.
pub trait AbsDelta<RHS = Self> {
    /// The type of the delta.
    type Delta;

    /// Gets the absolute delta between `self` and `rhs`.
    fn abs_delta(self, rhs: RHS) -> Self::Delta;
}

macro_rules! unsigned_impl {
    ($($type:ty) *) => {
        $(
            impl AbsDelta for $type {
                type Delta = $type;

                fn abs_delta(self, rhs: Self) -> Self::Delta {
                    if self > rhs { self - rhs } else { rhs - self }
                }
            }
        )*
    };
}

unsigned_impl!(u64 U48 u32 U24 u16 u8 usize);

macro_rules! signed_impl {
    ($($type:ty => $delta_type:ty), *) => {
        $(
            impl AbsDelta for $type {
                type Delta = $delta_type;

                fn abs_delta(self, rhs: Self) -> Self::Delta {
                    let (min, max) = if self > rhs { (rhs, self) } else { (self, rhs) };
        
                    let (result, overflowed) = (max as $delta_type).overflowing_sub(min as $delta_type);
                    if overflowed {
                        <$delta_type>::max_value()    
                    } else {
                        result
                    }
                }
            }
        )*
    };
}

signed_impl!(i64 => u64, i32 => u32, i16 => u16, i8 => u8, isize => usize);

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::u48::U48;
    use primitives::u24::U24;

    macro_rules! abs_delta_tests {
        ($($name:ident: $values:expr => $result:expr), *) => {
            $(
                #[test]
                fn $name() {
                    let (left, right) = $values;
                    assert_eq!(left.abs_delta(right), $result);
                }
            )*
        };
    }

    abs_delta_tests!(
        maximum_i64_descending_range_delta: (i64::max_value(), i64::min_value()) => u64::max_value(),
        maximum_i64_ascending_range_delta: (i64::min_value(), i64::max_value()) => u64::max_value(),
        i64_descending_range_delta: (6i64, 5i64) => 1u64,
        i64_ascending_range_delta: (5i64, 6i64) => 1u64,
        
        maximum_i32_descending_range_delta: (i32::max_value(), i32::min_value()) => u32::max_value(),
        maximum_i32_ascending_range_delta: (i32::min_value(), i32::max_value()) => u32::max_value(),
        i32_descending_range_delta: (6i32, 5i32) => 1u32,
        i32_ascending_range_delta: (5i32, 6i32) => 1u32,
        
        maximum_i16_descending_range_delta: (i16::max_value(), i16::min_value()) => u16::max_value(),
        maximum_i16_ascending_range_delta: (i16::min_value(), i16::max_value()) => u16::max_value(),
        i16_descending_range_delta: (6i16, 5i16) => 1u16,
        i16_ascending_range_delta: (5i16, 6i16) => 1u16,
        
        maximum_i8_descending_range_delta: (i8::max_value(), i8::min_value()) => u8::max_value(),
        maximum_i8_ascending_range_delta: (i8::min_value(), i8::max_value()) => u8::max_value(),
        i8_descending_range_delta: (6i8, 5i8) => 1u8,
        i8_ascending_range_delta: (5i8, 6i8) => 1u8,

        maximum_isize_descending_range_delta: (isize::max_value(), isize::min_value()) => usize::max_value(),
        maximum_isize_ascending_range_delta: (isize::min_value(), isize::max_value()) => usize::max_value(),
        isize_descending_range_delta: (6isize, 5isize) => 1usize,
        isize_ascending_range_delta: (5isize, 6isize) => 1usize,

        maximum_u64_descending_range_delta: (u64::max_value(), u64::min_value()) => u64::max_value(),
        maximum_u64_ascending_range_delta: (u64::min_value(), u64::max_value()) => u64::max_value(),
        u64_descending_range_delta: (6u64, 5u64) => 1u64,
        u64_ascending_range_delta: (5u64, 6u64) => 1u64,

        maximum_u48_descending_range_delta: (U48::max_value(), U48::min_value()) => U48::max_value(),
        maximum_u48_ascending_range_delta: (U48::min_value(), U48::max_value()) => U48::max_value(),
        u48_descending_range_delta: (U48::from(6u32), U48::from(5u32)) => U48::from(1u32),
        u48_ascending_range_delta: (U48::from(5u32), U48::from(6u32)) => U48::from(1u32),
        
        maximum_u32_descending_range_delta: (u32::max_value(), u32::min_value()) => u32::max_value(),
        maximum_u32_ascending_range_delta: (u32::min_value(), u32::max_value()) => u32::max_value(),
        u32_descending_range_delta: (6u32, 5u32) => 1u32,
        u32_ascending_range_delta: (5u32, 6u32) => 1u32,

        maximum_u24_descending_range_delta: (U24::max_value(), U24::min_value()) => U24::max_value(),
        maximum_u24_ascending_range_delta: (U24::min_value(), U24::max_value()) => U24::max_value(),
        u24_descending_range_delta: (U24::from(6u16), U24::from(5u16)) => U24::from(1u16),
        u24_ascending_range_delta: (U24::from(5u16), U24::from(6u16)) => U24::from(1u16),

        maximum_u16_descending_range_delta: (u16::max_value(), u16::min_value()) => u16::max_value(),
        maximum_u16_ascending_range_delta: (u16::min_value(), u16::max_value()) => u16::max_value(),
        u16_descending_range_delta: (6u16, 5u16) => 1u16,
        u16_ascending_range_delta: (5u16, 6u16) => 1u16,
        
        maximum_u8_descending_range_delta: (u8::max_value(), u8::min_value()) => u8::max_value(),
        maximum_u8_ascending_range_delta: (u8::min_value(), u8::max_value()) => u8::max_value(),
        u8_descending_range_delta: (6u8, 5u8) => 1u8,
        u8_ascending_range_delta: (5u8, 6u8) => 1u8,

        maximum_usize_descending_range_delta: (usize::max_value(), usize::min_value()) => usize::max_value(),
        maximum_usize_ascending_range_delta: (usize::min_value(), usize::max_value()) => usize::max_value(),
        usize_descending_range_delta: (6usize, 5usize) => 1usize,
        usize_ascending_range_delta: (5usize, 6usize) => 1usize
    );
}

