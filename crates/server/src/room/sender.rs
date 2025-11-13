use tokio::sync::mpsc;

use crate::messages::RoomMessage;

#[derive(Clone)]
pub struct RoomSender(mpsc::UnboundedSender<RoomMessage>);

impl RoomSender {
    pub fn new(sender: mpsc::UnboundedSender<RoomMessage>) -> Self {
        Self(sender)
    }

    pub fn send(&self, message: impl Into<RoomMessage>) {
        self.0.send(message.into()).ok();
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}
