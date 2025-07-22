use crate::{in_game::InGame, lobby::Lobby};

pub enum State {
    Lobby(Lobby),
    InGame(InGame),
}

impl Default for State {
    fn default() -> Self {
        Self::Lobby(Default::default())
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

    pub fn try_in_game(&mut self) -> anyhow::Result<&mut InGame> {
        if let Self::InGame(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected in game"))
        }
    }
}
