use rand::{seq::SliceRandom, thread_rng};
use std::sync::OnceLock;

pub static GLOBAL: OnceLock<GlobalData> = OnceLock::new();

#[derive(Debug)]
pub struct GlobalData {
    words: Vec<&'static str>,
    prompts: Vec<&'static str>,
}

impl GlobalData {
    pub fn new() -> Self {
        Self {
            words: include_str!("./static/words_alpha.txt").lines().collect(),
            prompts: include_str!("./static/prompts.txt").split(',').collect(),
        }
    }

    pub fn random_prompt(&self) -> &str {
        self.prompts.choose(&mut thread_rng()).unwrap()
    }

    pub fn random_anagram(&self) -> &str {
        loop {
            let anagram = *self.words.choose(&mut thread_rng()).unwrap();

            if anagram.len() == 6 {
                break anagram;
            }
        }
    }

    pub fn is_valid(&self, word: &str) -> bool {
        self.words.contains(&word)
    }
}
