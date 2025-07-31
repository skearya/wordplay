use crate::{
    game::Game,
    lobby::Lobby,
    room::{general::messages::ServerState, handler::Handler},
};

pub enum State {
    Lobby(Lobby),
    InGame(Game),
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
        }
    }

    pub fn end(&mut self) {
        match self {
            Self::Lobby(lobby) => lobby.end(),
            Self::InGame(in_game) => in_game.end(),
        }
    }
}
