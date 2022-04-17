use fltk::app::Sender;

use crate::gui::message_event::MessageEvent;

use super::{emoji_searcher::EmojiSearcher, searcher::Searcher};

pub struct SearchManager {
    current_searcher: Option<Box<dyn Searcher>>,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            current_searcher: None,
        }
    }

    pub fn search(&mut self, pattern: String, sender: Sender<MessageEvent>) {
        self.current_searcher = Self::find_searcher(&pattern);

        if let Some(searcher) = &mut self.current_searcher {
            searcher.search(pattern, sender);
        }
    }

    pub fn execute(&mut self, value: String) {
        if let Some(searcher) = &mut self.current_searcher {
            searcher.execute(value)
        }
    }

    fn find_searcher(pattern: &str) -> Option<Box<dyn Searcher>> {
        let searchers: Vec<Box<dyn Searcher>> = vec![Box::new(EmojiSearcher::new())];

        searchers
            .into_iter()
            .find(|searcher| searcher.handles(&pattern))
    }
}
