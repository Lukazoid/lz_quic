use errors::*;
use packets::PacketNumber;
use std::collections::HashSet;
use std::ops::Range;
use lz_diet::{Diet, Iter as DietIter};
use std::mem;
use std::borrow::Cow;
use binary_tree::{BinaryTree, Node, WalkAction};
use std::cmp;

#[derive(Debug, Clone, Default)]
pub struct PacketHistory {
    seen_packet_ranges: Diet<PacketNumber>,
    forgotten_up_to: Option<PacketNumber>,
}

pub struct PacketRangesIterator<'a> {
    diet_iter: DietIter<'a, PacketNumber>,
}

impl<'a> Iterator for PacketRangesIterator<'a> {
    type Item = Range<PacketNumber>;

    fn next(&mut self) -> Option<Self::Item> {
        self.diet_iter.next()
            .map(|range|range.clone().into())
    }
}

impl PacketHistory {
    pub fn new() -> Self {
        Default::default()
    }

    /// Attempts to push a `PacketNumber` returning whether the `PacketNumber` is new and was successfully pushed.
    ///
    /// # Returns
    /// Whether the packet was pushed successfully.
    pub fn push_packet_number(&mut self, packet_number: PacketNumber) -> bool {
        self.seen_packet_ranges.insert(packet_number)
    }

    pub fn ignore_packets_up_to_including(&mut self, packet_number: PacketNumber) {
        self.forgotten_up_to = Some(self.forgotten_up_to.map_or(packet_number, |f| cmp::max(f, packet_number)));

        let (_, greater) = self.seen_packet_ranges.split(Cow::Owned(packet_number));

        self.seen_packet_ranges = greater;
    }

    pub fn is_duplicate(&self, packet_number : PacketNumber) -> bool {
        if self.forgotten_up_to.map_or(false, |f| packet_number <= f) {
            true
        } else {
            self.seen_packet_ranges.contains(&packet_number)
        }
    }

    pub fn received_ranges<'a>(&'a self) -> PacketRangesIterator<'a> {
        PacketRangesIterator {
            diet_iter: self.seen_packet_ranges.iter()
        }
    }

    pub fn highest_range(&self) -> Option<Range<PacketNumber>> {
        self.seen_packet_ranges.root()
        .and_then(|r|{
            let mut max_interval = None;
            r.walk(|n| {
                max_interval = Some(n.value().clone());
                WalkAction::Right
            });

            max_interval.map(|interval|interval.into())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_duplicate_returns_false_for_empty() {
        let packet_history = PacketHistory::new();

        assert_eq!(packet_history.is_duplicate(5.into()), false);
    }

    #[test]
    fn is_duplicate_returns_true_for_received() {
        let mut packet_history = PacketHistory::new();

        packet_history.push_packet_number(5.into());

        assert!(packet_history.is_duplicate(5.into()));
    }

    #[test]
    fn is_duplicate_returns_true_for_ignored() {
        let mut packet_history = PacketHistory::new();

        packet_history.push_packet_number(5.into());
        packet_history.ignore_packets_up_to_including(200.into());

        assert!(packet_history.is_duplicate(5.into()));
    }

    #[test]
    fn ignore_packets_up_to_including_does_nothing_when_ignoring_already_ignored() {
        let mut packet_history = PacketHistory::new();

        packet_history.push_packet_number(5.into());
        packet_history.ignore_packets_up_to_including(200.into());
        packet_history.ignore_packets_up_to_including(4.into());

        assert!(packet_history.is_duplicate(5.into()));
    }
}