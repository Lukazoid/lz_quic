use std::cmp::{Ord, Ordering, PartialOrd};
use std::ops::{Deref, DerefMut};

/// Reverses the comparison of the inner value.
#[derive(Debug, Eq, PartialEq)]
pub struct RevOrd<T>(pub T);

impl<T: PartialOrd> PartialOrd for RevOrd<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl<T: Ord> Ord for RevOrd<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

impl<T> Deref for RevOrd<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RevOrd<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::RevOrd;

    #[test]
    fn cmp_test() {
        assert!(RevOrd(2) < RevOrd(1));
        assert!(RevOrd(1) > RevOrd(2));
    }
}
