#[derive(Debug, Default)]
pub struct FlowControl {
    max: u64,
    used: u64,
}

impl FlowControl {
    pub fn new() -> Self {
        Self::default()
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
