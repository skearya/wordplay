use uuid::Uuid;

pub trait ClientMessenger<Msg> {
    fn send(&self, uuid: Uuid, message: Msg);
    fn broadcast(&self, message: Msg);
    fn broadcast_except(&self, exclude: Uuid, message: Msg);
}

/// Creates an anonymous implementation of `ClientMessenger` that can **only** send sub-enums of `ServerMessage`.
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
        use crate::room::messenger::ClientMessenger;

        struct SubmessengerImpl<'a, T: ClientMessenger<$server_type>>(&'a mut T);

        impl<T: ClientMessenger<$server_type>> ClientMessenger<$subtype> for SubmessengerImpl<'_, T> {
            fn send(&self, uuid: Uuid, message: $subtype) {
                self.0.send(uuid, $server_type::$variant(message))
            }

            fn broadcast(&self, message: $subtype) {
                self.0.broadcast($server_type::$variant(message))
            }

            fn broadcast_except(&self, except: Uuid, message: $subtype) {
                self.0.broadcast_except(except, $server_type::$variant(message))
            }
        }

        SubmessengerImpl($messenger)
    }};
}

pub(crate) use client_submessenger;
