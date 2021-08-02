use crate::packet::Packet;
use async_trait::async_trait;

#[async_trait]
pub trait PacketStream {
    async fn next(&mut self) -> Option<Packet>;
}
