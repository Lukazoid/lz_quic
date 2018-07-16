use conv::{UnwrapOrSaturate, ValueFrom, ValueInto};
use std::cmp;
use std::mem;

#[derive(Debug, Default)]
pub struct FlowControl {
    max: u64,
    used: u64,
}

impl FlowControl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn take(first: &mut FlowControl, second: &mut FlowControl, amount: usize) -> usize {
        // if amount is bigger than a u64 we will simply take u64 bytes
        // it's likely the remaining will reduce this amount anyway
        let amount: u64 = u64::value_from(amount).unwrap_or_saturate();

        let available = cmp::min(first.remaining(), second.remaining());

        let taken = cmp::min(amount, available);

        first.used += taken;
        second.used += taken;

        let taken: usize = taken.value_into().expect("we know taken must be amount (which is a valid usize) or less so conversion to usize cannot ever fail");

        taken
    }

    pub fn with_initial_max(max: u64) -> Self {
        Self { max, used: 0 }
    }

    pub fn advance_max(&mut self, max: u64) -> bool {
        if max > self.max {
            self.max = max;
            true
        } else {
            false
        }
    }

    pub fn max(&self) -> u64 {
        self.max
    }

    pub fn used(&self) -> u64 {
        self.used
    }

    pub fn remaining(&self) -> u64 {
        self.max - self.used
    }
}
