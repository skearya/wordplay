use tokio::sync::mpsc;

use crate::{
    game::{
        anagrams::messages::AnagramsMessage, messages::GameMessage,
        word_bomb::messages::WordBombMessage,
    },
    lobby::messages::LobbyMessage,
    messages::RoomMessage,
};

#[derive(Clone)]
pub struct RoomSender(mpsc::UnboundedSender<RoomMessage>);

impl RoomSender {
    pub fn new(sender: mpsc::UnboundedSender<RoomMessage>) -> Self {
        Self(sender)
    }

    pub fn send(&self, message: RoomMessage) {
        self.0.send(message).ok();
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }
}

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
    (name: $name:ident, inherits: $sender:ty, $root:ident :: $variant:ident( $subtype:ty )) => {
        #[derive(Clone)]
        pub struct $name($sender);

        impl $name {
            pub fn new(sender: $sender) -> Self {
                Self(sender)
            }

            pub fn send(&self, message: $subtype) {
                self.0.send($root::$variant(message));
            }
        }
    };
    (name: $name:ident, $room_type:ident :: $variant:ident( $subtype:ty )) => {
        room_submessenger!(name: $name, inherits: RoomSender, $room_type::$variant($subtype));
    };
}

room_submessenger!(name: LobbySender, RoomMessage::Lobby(LobbyMessage));

room_submessenger!(name: GameSender, RoomMessage::Game(GameMessage));

room_submessenger!(
    name: WordBombSender,
    inherits: GameSender,
    GameMessage::WordBomb(WordBombMessage)
);

room_submessenger!(
    name: AnagramsSender,
    inherits: GameSender,
    GameMessage::Anagrams(AnagramsMessage)
);
