use uuid::Uuid;

pub trait ClientMessenger<Msg> {
    fn send(&self, uuid: Uuid, message: Msg);
    fn broadcast(&self, message: Msg);
    fn broadcast_except(&self, exclude: Uuid, message: Msg);
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

        SubmessengerImpl($messenger)
    }};
}

pub(crate) use client_submessenger;

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
