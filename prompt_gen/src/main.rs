use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::{fs, str};

const MIN_WPP: u32 = 500;

fn main() {
    let words: Vec<&str> = include_str!("./words_alpha.txt").lines().collect();

    let prompts: HashSet<&str> = words
        .iter()
        .filter_map(|word| get_all_slices(word))
        .flatten()
        .collect();

    let prompt_counts: HashMap<&&str, u32> = prompts
        .par_iter()
        .map(|prompt| {
            (
                prompt,
                words.iter().fold(
                    0,
                    |acc, word| if word.contains(prompt) { acc + 1 } else { acc },
                ),
            )
        })
        .collect();

    fs::write(
        "./output/prompts.txt",
        prompt_counts
            .iter()
            .filter(|(_, &count)| count >= MIN_WPP)
            .map(|(&&word, _)| word)
            .collect::<Vec<&str>>()
            .join(","),
    )
    .expect("failed to write to file");
}

fn get_all_slices(word: &str) -> Option<Vec<&str>> {
    if word.len() < 2 {
        return None;
    }

    Some(
        (2..=3)
            .flat_map(|size| {
                word.as_bytes()
                    .chunks_exact(size)
                    .map(|chunk| str::from_utf8(chunk).unwrap())
                    .collect::<Vec<&str>>()
            })
            .collect(),
    )
}
