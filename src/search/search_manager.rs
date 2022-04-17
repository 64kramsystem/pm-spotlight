use fltk::app::Sender;

use crate::gui::message_event::MessageEvent;

use super::{emoji_searcher::EmojiSearcher, searcher::Searcher};

pub struct SearchManager {
    current_searcher: Option<Box<dyn Searcher>>,
    // This type performs dumb id generation, but no checks. The reason is that checks must be performed
    // by the App type (e.g. display or not the entries sent from a search), so it's cleaner to perform
    // all of them there.
    //
    current_search_id: u32,
}

impl SearchManager {
    pub fn new() -> Self {
        Self {
            current_searcher: None,
            current_search_id: 0,
        }
    }

    pub fn search(&mut self, pattern: String, sender: Sender<MessageEvent>) -> u32 {
        // Increase anyway. If no searchers are found, it's still meaningful that other messages should
        // be ignored.
        //
        self.current_search_id += 1;

        self.current_searcher = Self::find_searcher(&pattern);

        if let Some(searcher) = &mut self.current_searcher {
            searcher.search(pattern, sender, self.current_search_id);
        }

        self.current_search_id
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
