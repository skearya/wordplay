use rand::{seq::SliceRandom, thread_rng};
use std::sync::OnceLock;

pub static GLOBAL: OnceLock<GlobalData> = OnceLock::new();

#[derive(Debug)]
pub struct GlobalData<'a> {
    words: Vec<&'a str>,
    prompts: Vec<&'a str>,
}

impl GlobalData<'_> {
    pub fn new() -> Self {
        Self {
            words: include_str!("./static/words_alpha.txt").lines().collect(),
            prompts: include_str!("./static/prompts.txt").split(',').collect(),
        }
    }

    pub fn random_prompt(&self) -> String {
        (*self.prompts.choose(&mut thread_rng()).unwrap()).to_string()
    }

    pub fn is_valid(&self, word: &str) -> bool {
        self.words.contains(&word)
    }
}
