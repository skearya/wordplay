use uuid::Uuid;

pub trait ClientMessenger<Msg> {
    fn send(&self, uuid: Uuid, message: Msg);
    fn broadcast(&self, message: Msg);
    fn broadcast_except(&self, exclude: Uuid, message: Msg);
}

pub trait ClientUtils {
    fn get(&self, uuid: Uuid) -> Option<&Client>;
    fn iter(&self) -> std::collections::hash_map::Iter<'_, Uuid, Client>;
    fn is_empty(&self) -> bool;
}

pub trait ClientUtilsMut {
    fn add(&mut self, uuid: Uuid, client: Client);
    fn remove(&mut self, uuid: Uuid, socket: Uuid);
    fn get_mut(&mut self, uuid: Uuid) -> Option<&mut Client>;
}

// `Send + 'static` needed to allow `RoomMessenger` to be used in futures.
pub trait RoomMessenger<Msg>: Send + 'static {
    fn send(&self, message: Msg);
}

/// Creates an implementation of `ClientMessenger` that can **only** send sub-enums of `ServerMessage`.
///
/// ### Usage
/// ```
/// client_submessenger!(&Clients, ServerMessage::Variant(SubEnum))
/// ```
///
/// ### Example
/// ```
/// // Send messages of `ServerMessage::Lobby` variant, which hold `ServerLobby` enums.
/// let sub = client_submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby));
///
/// // Equivalent to `clients.broadcast(ServerMessage::Lobby(ServerLobby::Ready { uuid }))`
/// sub.broadcast(ServerLobby::Ready { uuid });
/// ```
macro_rules! client_submessenger {
    ($messenger:expr, $server_type:ident :: $variant:ident( $subtype:ty )) => {{
        use crate::room::{
            clients::Client,
            messenger::{ClientMessenger, ClientUtils},
        };

        struct SubmessengerImpl<'a, T: ClientMessenger<$server_type>>(&'a T);

        impl<T: ClientMessenger<$server_type>> ClientMessenger<$subtype> for SubmessengerImpl<'_, T> {
            fn send(&self, uuid: Uuid, message: $subtype) {
                self.0.send(uuid, $server_type::$variant(message));
            }

            fn broadcast(&self, message: $subtype) {
                self.0.broadcast($server_type::$variant(message));
            }

            fn broadcast_except(&self, except: Uuid, message: $subtype) {
                self.0.broadcast_except(except, $server_type::$variant(message));
            }
        }

        impl<T: ClientMessenger<$server_type> + ClientUtils> ClientUtils for SubmessengerImpl<'_, T> {
            fn get(&self, uuid: Uuid) -> Option<&Client> {
                self.0.get(uuid)
            }

            fn iter(&self) -> std::collections::hash_map::Iter<'_, Uuid, Client> {
                self.0.iter()
            }

            fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        SubmessengerImpl($messenger)
    }};
}

pub(crate) use client_submessenger;

/// Copy of `client_submessenger` but with the `ClientUtilsMut` trait.
macro_rules! client_submessenger_mut {
    ($messenger:expr, $server_type:ident :: $variant:ident( $subtype:ty )) => {{
        use crate::room::{
            clients::Client,
            messenger::{ClientMessenger, ClientUtils, ClientUtilsMut},
        };

        struct SubmessengerImpl<'a, T: ClientMessenger<$server_type>>(&'a mut T);

        impl<T: ClientMessenger<$server_type>> ClientMessenger<$subtype> for SubmessengerImpl<'_, T> {
            fn send(&self, uuid: Uuid, message: $subtype) {
                self.0.send(uuid, $server_type::$variant(message));
            }

            fn broadcast(&self, message: $subtype) {
                self.0.broadcast($server_type::$variant(message));
            }

            fn broadcast_except(&self, except: Uuid, message: $subtype) {
                self.0.broadcast_except(except, $server_type::$variant(message));
            }
        }

        impl<T: ClientMessenger<$server_type> + ClientUtils> ClientUtils for SubmessengerImpl<'_, T> {
            fn get(&self, uuid: Uuid) -> Option<&Client> {
                self.0.get(uuid)
            }

            fn iter(&self) -> std::collections::hash_map::Iter<'_, Uuid, Client> {
                self.0.iter()
            }

            fn is_empty(&self) -> bool {
                self.0.is_empty()
            }
        }

        impl<T: ClientMessenger<$server_type> + ClientUtilsMut> ClientUtils for SubmessengerImpl<'_, T> {
            fn add(&mut self, uuid: Uuid, client: Client) {
                self.0.add(uuid, client)
            }

            fn remove(&mut self, uuid: Uuid, socket: Uuid) {
                self.0.remove(uuid, socket)
            }

            fn get_mut(&mut self, uuid: Uuid) -> Option<&mut Client> {
                self.0.get_mut(uuid, uuid)
            }
        }

        SubmessengerImpl($messenger)
    }};
}

pub(crate) use client_submessenger_mut;

/// Creates an implementation of `RoomMessenger` that can **only** send sub-enums of `RoomMessage`.
///
/// ### Usage
/// ```
/// room_submessenger!(&Clients, RoomMessage::Variant(SubEnum))
/// ```
///
/// ### Example
/// ```
/// // Send messages of `ServerMessage::Lobby` variant, which hold `ServerLobby` enums.
/// let sub = client_submessenger!(&self.clients, RoomMessage::Lobby(LobbyMessage));
///
/// // Equivalent to `clients.broadcast(RoomMessage::Lobby(LobbyMessage::GameStart))`
/// sub.broadcast(LobbyMessage::GameStart);
/// ```
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

pub(crate) use room_submessenger;

use crate::room::clients::Client;
