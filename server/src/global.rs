use rand::{seq::SliceRandom, thread_rng};
use std::sync::LazyLock;

pub static GLOBAL: LazyLock<GlobalData> = LazyLock::new(GlobalData::new);

pub struct GlobalData {
    pub words: Vec<&'static str>,
    pub prompts: Prompts,
}

impl GlobalData {
    pub fn new() -> Self {
        Self {
            words: include_str!("./static/words_alpha.txt").lines().collect(),
            prompts: Prompts::new(),
        }
    }

    pub fn is_valid(&self, word: &str) -> bool {
        self.words.binary_search(&word).is_ok()
    }

    pub fn random_anagram(&self) -> (&str, String) {
        loop {
            let anagram = *self.words.choose(&mut thread_rng()).unwrap();

            if anagram.len() == 6 {
                let mut chars: Vec<char> = anagram.chars().collect();
                chars.shuffle(&mut thread_rng());

                break (anagram, chars.into_iter().collect());
            }
        }
    }
}

pub struct Prompts {
    prompts: Vec<&'static str>,
    wpp_indexes: Vec<(usize, usize)>,
}

impl Prompts {
    fn new() -> Self {
        let list = include_str!("./static/prompts.txt")
            .lines()
            .filter_map(|line| line.split_once(':'));

        let prompts: Vec<&str> = list
            .clone()
            .flat_map(|(_, prompts)| prompts.split(','))
            .collect();

        let wpp_indexes: Vec<(usize, usize)> = list
            .filter_map(|(wpp, prompts)| wpp.parse().ok().map(|wpp| (wpp, prompts)))
            .filter_map(|(wpp, line_prompts)| {
                line_prompts
                    .split(',')
                    .next()
                    .and_then(|first| prompts.iter().position(|&prompt| prompt == first))
                    .map(|index| (wpp, index))
            })
            .collect();

        Self {
            prompts,
            wpp_indexes,
        }
    }

    pub fn random_prompt(&self, min_wpp: usize) -> &str {
        let (_, closest_index) = self
            .wpp_indexes
            .iter()
            .min_by_key(|(index, _)| index.abs_diff(min_wpp))
            .unwrap();

        self.prompts[*closest_index..]
            .choose(&mut thread_rng())
            .unwrap()
    }
}
