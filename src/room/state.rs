use crate::{in_game::InGame, lobby::Lobby};

pub enum State {
    Lobby(Lobby),
    InGame(InGame),
}

// TODO: all of this can be done with derive_more i think
impl State {
    pub fn name(&self) -> &'static str {
        match self {
            State::Lobby(_) => "lobby",
            State::InGame(_) => "in game",
        }
    }

    pub fn try_lobby(&mut self) -> anyhow::Result<&mut Lobby> {
        if let Self::Lobby(lobby) = self {
            Ok(lobby)
        } else {
            Err(anyhow::anyhow!(
                "expected to be in lobby, in {}",
                self.name()
            ))
        }
    }

    pub fn try_in_game(&mut self) -> anyhow::Result<&mut InGame> {
        if let Self::InGame(in_game) = self {
            Ok(in_game)
        } else {
            Err(anyhow::anyhow!(
                "expected to be in word bomb, in {}",
                self.name()
            ))
        }
    }
}
