use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::{fs, str};

fn main() {
    let words: Vec<&str> = include_str!("../../server/src/static/words_alpha.txt")
        .lines()
        .collect();

    let prompts: HashSet<&str> = words
        .iter()
        .filter_map(|word| get_all_slices(word))
        .flatten()
        .collect();

    // Vec<(prompt, amount of times its in other words)>
    let prompt_counts: Vec<(&str, usize)> = prompts
        .par_iter()
        .map(|&prompt| {
            (
                prompt,
                words.iter().filter(|word| word.contains(prompt)).count(),
            )
        })
        .collect();

    // HashMap<amount of times prompt used in other words, prompts>
    let mut prompt_counts_map: HashMap<usize, Vec<&str>> = HashMap::new();

    for (prompt, count) in prompt_counts {
        prompt_counts_map.entry(count).or_default().push(prompt);
    }

    // Vec<(amount of times prompt used in other words, prompts)> | collected to vec for sorting
    let mut prompt_counts: Vec<(usize, Vec<&str>)> = prompt_counts_map.into_iter().collect();
    prompt_counts.sort_by_key(|pair| pair.0);

    let lines: String = prompt_counts
        .into_iter()
        .map(|(count, words)| format!("{count}:{}", words.join(",")))
        .collect::<Vec<String>>()
        .join("\n");

    fs::write("../server/src/static/prompts.txt", lines).expect("failed to write to file");
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
