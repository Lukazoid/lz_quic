#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ConnectionTerminationMode {
    /// A packet is sent to the remote endpoint indicating the connection has been closed.
    Explicit,

    /// No packet is sent to the remote endpoint and instead the remote endpoint assumes the connection has
    /// terminated after an idle timeout.
    Implicit,
}
