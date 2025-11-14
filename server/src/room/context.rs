use crate::{
    messages::RoomSettings,
    room::{clients::Clients, sender::RoomSender},
};

pub struct Context<'a> {
    pub room: &'a RoomSender,
    pub clients: &'a mut Clients,
    pub settings: &'a mut RoomSettings,
}

impl<'a> Context<'a> {
    pub fn new(
        room: &'a RoomSender,
        clients: &'a mut Clients,
        settings: &'a mut RoomSettings,
    ) -> Self {
        Self {
            room,
            clients,
            settings,
        }
    }
}
