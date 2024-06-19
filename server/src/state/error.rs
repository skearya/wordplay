use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("room `{room}` not found")]
    RoomNotFound { room: String },
    #[error("room was not in state `{state}` ")]
    InvalidState { state: &'static str },
    #[error(transparent)]
    Room(#[from] RoomError),
    #[error(transparent)]
    WordBomb(#[from] WordBombError),
    #[error(transparent)]
    Anagrams(#[from] AnagramsError),
}

#[derive(Error, Debug)]
pub enum RoomError {
    #[error("chat message was too long")]
    ChatMessageTooLong,
    #[error("client couldn't be found while removing")]
    CouldntFindClientToRemove,
    #[error("client's socket uuids did not match up while removing")]
    SocketUuidMismatchWhileRemoving,
}

#[derive(Error, Debug)]
pub enum AnagramsError {
    #[error("player's guess was too long")]
    GuessTooLong,
    #[error("spectator(?) tried playing")]
    PlayerNotFound,
}

#[derive(Error, Debug)]
pub enum WordBombError {
    #[error("player's input was too long")]
    InputTooLong,
    #[error("player's guess was too long")]
    GuessTooLong,
    #[error("spectator(?) tried playing")]
    PlayerNotFound,
    #[error("client tried playing out of turn")]
    OutOfTurn,
    #[error("can't update turn because nobody/nobody else is alive")]
    NoPlayersAlive,
}
