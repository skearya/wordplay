use tokio::sync::mpsc;

use crate::{messages::RoomMessage, room::messenger::RoomMessenger};

#[derive(Clone)]
pub struct RoomSender(mpsc::UnboundedSender<RoomMessage>);

impl RoomSender {
    pub fn new(sender: mpsc::UnboundedSender<RoomMessage>) -> Self {
        Self(sender)
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}

impl RoomMessenger<RoomMessage> for RoomSender {
    fn send(&self, message: RoomMessage) {
        self.0.send(message).ok();
    }
}
