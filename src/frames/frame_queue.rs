use bytes::Bytes;
use frames::Frame;
use protocol::StreamId;
use std::collections::VecDeque;
use std::time::Instant;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
struct PendingFrame {
    pub stream_id: StreamId,
    pub frame: Frame,
    pub last_transmitted_at: Option<Instant>,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FrameQueue {
    pending_frames: VecDeque<PendingFrame>,
}

impl FrameQueue {
    pub fn enqueue(&mut self, stream_id: StreamId, frame: Frame) {
        self.pending_frames.push_back(PendingFrame {
            stream_id,
            frame,
            last_transmitted_at: None,
        })
    }
}
