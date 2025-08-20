use crate::{game::Game, lobby::Lobby, messages::ServerState, room::handler::Handler};

pub enum State {
    Lobby(Lobby),
    InGame(Game),
    /// Indicator that the room task should end.
    Ended,
}

impl Default for State {
    fn default() -> Self {
        Self::Lobby(Lobby::default())
    }
}

impl State {
    pub fn try_lobby(&mut self) -> anyhow::Result<&mut Lobby> {
        if let Self::Lobby(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected lobby",))
        }
    }

    pub fn try_in_game(&mut self) -> anyhow::Result<&mut Game> {
        if let Self::InGame(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected in game"))
        }
    }

    pub fn state(&self) -> ServerState {
        match self {
            Self::Lobby(lobby) => ServerState::Lobby(lobby.state()),
            Self::InGame(in_game) => ServerState::Game(in_game.state()),
            Self::Ended => unreachable!(),
        }
    }

    pub fn end(&mut self) {
        match self {
            Self::Lobby(lobby) => lobby.end(),
            Self::InGame(in_game) => in_game.end(),
            Self::Ended => unreachable!(),
        }
    }
}
