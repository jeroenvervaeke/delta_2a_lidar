use crate::packet::Packet;
use async_trait::async_trait;

/// Abstraction over a packet stream
///
/// Accept this abstraction instead of using `Lidar` directly.
/// This way you can use both a real lidar and mocked lidar (or a measurements file)
#[async_trait]
pub trait PacketStream {
    /// Reads the next lidar package.
    /// Returns None if the stream has ended.
    async fn next(&mut self) -> Option<Packet>;
}
