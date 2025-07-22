use std::marker::PhantomData;

use uuid::Uuid;

pub trait ClientMessenger<Msg> {
    fn send(&self, uuid: Uuid, message: Msg);
    fn broadcast(&self, message: Msg);
}

// `Send + 'static` needed to allow `RoomMessenger` to be used in futures.
pub trait RoomMessenger<Msg>: Send + 'static {
    fn send(&self, message: Msg);
}

pub struct Messenger<ClientMsg, RoomMsg, C, R>
where
    C: ClientMessenger<ClientMsg>,
    R: RoomMessenger<RoomMsg>,
{
    clients: C,
    room: R,

    // Dealing with unused type parameters.
    _phantom0: PhantomData<ClientMsg>,
    _phantom1: PhantomData<RoomMsg>,
}

impl<ClientMsg, RoomMsg, C, R> Messenger<ClientMsg, RoomMsg, C, R>
where
    C: ClientMessenger<ClientMsg>,
    R: RoomMessenger<RoomMsg>,
{
    pub fn new(clients: C, room: R) -> Self {
        Self {
            clients,
            room,
            _phantom0: PhantomData,
            _phantom1: PhantomData,
        }
    }
}

/// Creates an implementation of `Messenger` that can **only** send sub-enums of `ServerMessage`.
///
/// ### Usage
/// ```
/// submessenger!(&Clients, ServerMessage::Variant(SubEnum))
/// ```
///
/// ### Example
/// ```
/// // Send messages of `ServerMessage::Lobby` variant, which hold `ServerLobby` enums.
/// let sub = submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby));
///
/// // Equivalent to `clients.broadcast(ServerMessage::Lobby(ServerLobby::Ready { uuid }))`
/// sub.broadcast(ServerLobby::Ready { uuid });
/// ```
macro_rules! submessenger {
    ($clients:expr, ServerMessage::$server_variant:tt($server_subtype:ty), $room:expr, RoomMessage::$room_variant:tt($room_subtype:ty)) => {{
        struct ClientMessengerImpl<'a>(&'a Clients);

        impl ClientMessenger<$server_subtype> for ClientMessengerImpl<'_> {
            fn send(&self, uuid: Uuid, message: $server_subtype) {
                self.0.send(uuid, &ServerMessage::$server_variant(message));
            }

            fn broadcast(&self, message: $server_subtype) {
                self.0.broadcast(&ServerMessage::$server_variant(message));
            }
        }

        struct RoomMessengerImpl(mpsc::UnboundedSender<RoomMessage>);

        impl RoomMessenger<$room_subtype> for RoomMessengerImpl {
            fn send(&self, message: $room_subtype) {
                self.0
                    .send(RoomMessage::$room_variant(message))
                    .expect("room should not be closed");
            }
        }

        Messenger::new(
            ClientMessengerImpl($clients),
            RoomMessengerImpl($room.clone()),
        )
    }};
}

macro_rules! client_submessenger {
    ($messenger:expr, $server_type:ident :: $variant:ident( $subtype:ty )) => {{
        struct SubmessengerImpl<T: ClientMessenger<$server_type>>(T);

        impl<T: ClientMessenger<$server_type>> ClientMessenger<$subtype> for SubmessengerImpl<T> {
            fn send(&self, uuid: Uuid, message: $subtype) {
                self.0.send(uuid, $server_type::$variant(message));
            }

            fn broadcast(&self, message: $subtype) {
                self.0.broadcast($server_type::$variant(message));
            }
        }

        SubmessengerImpl($messenger)
    }};
}

macro_rules! room_submessenger {
    ($messenger:expr, $room_type:ident :: $variant:ident( $subtype:ty )) => {{
        struct RoomMessengerImpl<T: RoomMessenger<$room_type>>(T);

        impl<T: RoomMessenger<$room_type>> RoomMessenger<$subtype> for RoomMessengerImpl<T> {
            fn send(&self, message: $subtype) {
                self.0.send($room_type::$variant(message));
            }
        }

        RoomMessengerImpl($messenger)
    }};
}

macro_rules! submessenger2 {
    ($clients:expr => $server_type:tt::$server_variant:tt($server_subtype:ty), $room:expr => $room_type:tt::$room_variant:tt($room_subtype:ty)) => {{
        struct ClientMessengerImpl<T: ClientMessenger<$server_type>>(T);

        impl<T: ClientMessenger<$server_type>> ClientMessenger<$server_subtype> for ClientMessengerImpl<T> {
            fn send(&self, uuid: Uuid, message: $server_subtype) {
                self.0.send(uuid, $server_type::$server_variant(message));
            }
            fn broadcast(&self, message: $server_subtype) {
                self.0.broadcast($server_type::$server_variant(message));
            }
        }

        struct RoomMessengerImpl<T: RoomMessenger<$room_type>>(T);

        impl<T: RoomMessenger<$room_type>> RoomMessenger<$room_subtype> for RoomMessengerImpl<T> {
            fn send(&self, message: $room_subtype) {
                self.0.send($room_type::$room_variant(message));
            }
        }

        Messenger::new(
            ClientMessengerImpl($clients),
            RoomMessengerImpl($room.clone()),
        )
    }};
}

fn test(clients: Clients, room: RoomSender) {
    // todo split into client submmesanger and server submessenger
    // todo fix usages of messangers (state handlers)
    // todo move states (lobby, general, games) into /states
    // move games into in_game
    // think about how game ending messages get sent

    let in_game = submessenger2!(
        clients => ServerMessage::InGame(ServerInGame),
        room => RoomMessage::Lobby(LobbyMessage)
    );

    in_game
        .clients
        .broadcast(ServerInGame::WordBomb(ServerWordBomb::Input {
            input: "e".to_string(),
        }));

    let wbo = room_submessenger!(room, RoomMessage::Lobby(LobbyMessage));
}

pub(crate) use client_submessenger;
pub(crate) use room_submessenger;
pub(crate) use submessenger;

use crate::{
    games::word_bomb::messages::ServerWordBomb,
    in_game::messages::ServerInGame,
    lobby::messages::{LobbyMessage, ServerLobby},
    messages::{RoomMessage, ServerMessage},
    room::clients::{Clients, RoomSender},
};
