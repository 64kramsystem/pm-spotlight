use phf::phf_map;

use super::searcher::Searcher;

const EMOJI_PATTERNS: phf::Map<&str, &str> = phf_map! {
    "bald"       => "🦲",
    "party_face" => "🥳",
};

pub struct EmojiSearcher {}

impl EmojiSearcher {
    pub fn new() -> Self {
        Self {}
    }
}

impl Searcher for EmojiSearcher {
    fn handles(&self, pattern: &str) -> bool {
        pattern.starts_with(":")
    }

    fn search(&self, pattern: &str) -> Vec<String> {
        let pattern = pattern.chars().skip(1).collect::<String>();

        if pattern.len() > 0 {
            let matching_emojis = EMOJI_PATTERNS
                .into_iter()
                .filter_map(|(patterns, emoji)| {
                    if patterns.contains(&pattern) {
                        Some(emoji.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            matching_emojis
        } else {
            vec![]
        }
    }
}
