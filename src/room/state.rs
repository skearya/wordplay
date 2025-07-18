use crate::{games::word_bomb::WordBomb, lobby::Lobby};

pub enum State {
    Lobby(Lobby),
    WordBomb(WordBomb),
}

impl State {
    pub fn name(&self) -> &'static str {
        match self {
            State::Lobby(_) => "lobby",
            State::WordBomb(_) => "word bomb",
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

    pub fn try_word_bomb(&mut self) -> anyhow::Result<&mut WordBomb> {
        if let Self::WordBomb(word_bomb) = self {
            Ok(word_bomb)
        } else {
            Err(anyhow::anyhow!(
                "expected to be in word bomb, in {}",
                self.name()
            ))
        }
    }
}
