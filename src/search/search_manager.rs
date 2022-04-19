use fltk::app::Sender;

use crate::{config::config_manager::Config, gui::message_event::MessageEvent};

use super::{emoji_searcher::EmojiSearcher, file_searcher::FileSearcher, searcher::Searcher};

pub struct SearchManager {
    config: Config,
    current_searcher: Option<Box<dyn Searcher>>,
    // This type performs dumb id generation, but no checks. The reason is that checks must be performed
    // by the App type (e.g. display or not the entries sent from a search), so it's cleaner to perform
    // all of them there.
    //
    current_search_id: u32,
}

impl SearchManager {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            current_searcher: None,
            current_search_id: 0,
        }
    }

    pub fn search(&mut self, pattern: String, sender: Sender<MessageEvent>) -> u32 {
        // Increase anyway. If no searchers are found, it's still meaningful that other messages should
        // be ignored.
        //
        self.current_search_id += 1;

        if let Some(searcher) = &mut self.current_searcher {
            searcher.stop();
        }

        self.current_searcher = self.find_searcher(&pattern);

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

    pub fn alt_execute(&mut self, value: String) -> bool {
        if let Some(searcher) = &mut self.current_searcher {
            searcher.alt_execute(value)
        } else {
            false
        }
    }

    fn find_searcher(&self, pattern: &str) -> Option<Box<dyn Searcher>> {
        // WATCH OUT!! The ordering matters - specialized searchers must go first, since the file always
        // handles the pattern, and prevents the following ones from running.
        //
        let searchers: Vec<Box<dyn Searcher>> = vec![
            Box::new(EmojiSearcher::new()),
            Box::new(FileSearcher::new(self.config.clone())),
        ];

        searchers
            .into_iter()
            .find(|searcher| searcher.handles(&pattern))
    }
}
