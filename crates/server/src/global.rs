use std::sync::LazyLock;

use rand::seq::IndexedRandom;

static WORDS: LazyLock<Vec<&'static str>> = LazyLock::new(|| {
    let mut words: Vec<&'static str> = include_str!("res/words_alpha.txt").lines().collect();
    words.sort_unstable();

    words
});

static PROMPTS: LazyLock<Vec<(usize, Vec<&'static str>)>> = LazyLock::new(|| {
    include_str!("res/prompts.txt")
        .lines()
        .map(|line| line.split_once(':').unwrap())
        .map(|(wpp, prompts)| (wpp.parse().unwrap(), prompts.split(',').collect()))
        .collect()
});

pub fn init_globals() {
    LazyLock::force(&WORDS);
    LazyLock::force(&PROMPTS);
}

pub fn is_english(word: &str) -> bool {
    WORDS.binary_search(&word).is_ok()
}

pub fn random_prompt(min_wpp: usize) -> &'static str {
    let (index, _) = PROMPTS
        .iter()
        .enumerate()
        .min_by_key(|(_, (wpp, _))| wpp.abs_diff(min_wpp))
        .unwrap();

    let (_, prompts) = PROMPTS[index..].choose(&mut rand::rng()).unwrap();

    prompts.choose(&mut rand::rng()).unwrap()
}
