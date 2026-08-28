use std::{path::PathBuf, sync::Arc};

use crate::{
    config::config_manager::Config,
    helpers::desktop_integration::{DesktopIntegration, NativeDesktopIntegration},
};

use super::{
    emoji_searcher::EmojiSearcher,
    file_searcher::FileSearcher,
    searcher::{ExecutionAction, SearchResultSink, Searcher},
};

pub struct SearchManager {
    config: Config,
    desktop: Arc<dyn DesktopIntegration>,
    home_dir: Option<PathBuf>,
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
            desktop: Arc::new(NativeDesktopIntegration),
            home_dir: None,
            current_searcher: None,
            current_search_id: 0,
        }
    }

    pub fn with_dependencies(
        config: Config,
        desktop: Arc<dyn DesktopIntegration>,
        home_dir: PathBuf,
    ) -> Self {
        Self {
            config,
            desktop,
            home_dir: Some(home_dir),
            current_searcher: None,
            current_search_id: 0,
        }
    }

    pub fn search(&mut self, pattern: String, sink: Arc<dyn SearchResultSink>) -> u32 {
        // Increase anyway. If no searchers are found, it's still meaningful that other messages should
        // be ignored.
        //
        self.current_search_id += 1;

        if let Some(searcher) = &mut self.current_searcher {
            searcher.stop();
        }

        self.current_searcher = self.find_searcher(&pattern);

        if let Some(searcher) = &mut self.current_searcher {
            searcher.search(pattern, sink, self.current_search_id);
        }

        self.current_search_id
    }

    pub fn execute(&mut self, value: String) -> Result<ExecutionAction, String> {
        if let Some(searcher) = &mut self.current_searcher {
            searcher.execute(value)
        } else {
            Ok(ExecutionAction::Ignore)
        }
    }

    pub fn alt_execute(&mut self, value: String) -> Result<ExecutionAction, String> {
        if let Some(searcher) = &mut self.current_searcher {
            searcher.alt_execute(value)
        } else {
            Ok(ExecutionAction::Ignore)
        }
    }

    fn find_searcher(&self, pattern: &str) -> Option<Box<dyn Searcher>> {
        // WATCH OUT!! The ordering matters - specialized searchers must go first, since the file always
        // handles the pattern, and prevents the following ones from running.
        //
        let file_searcher = match &self.home_dir {
            Some(home_dir) => FileSearcher::with_home(
                self.config.clone(),
                Arc::clone(&self.desktop),
                home_dir.clone(),
            ),
            None => FileSearcher::new(self.config.clone(), Arc::clone(&self.desktop)),
        };

        let searchers: Vec<Box<dyn Searcher>> = vec![
            Box::new(EmojiSearcher::new(Arc::clone(&self.desktop))),
            Box::new(file_searcher),
        ];

        searchers
            .into_iter()
            .find(|searcher| searcher.handles(pattern))
    }
}
