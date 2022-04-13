use super::{emoji_searcher::EmojiSearcher, searcher::Searcher};

pub struct SearchersProvider {}

impl SearchersProvider {
    pub fn new() -> Self {
        Self {}
    }

    pub fn find_provider(&self, pattern: &str) -> Option<Box<dyn Searcher>> {
        let searchers: Vec<Box<dyn Searcher>> = vec![Box::new(EmojiSearcher::new())];

        searchers
            .into_iter()
            .find(|searcher| searcher.handles(&pattern))
    }
}
