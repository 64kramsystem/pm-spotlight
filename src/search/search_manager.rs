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
    desktop: Arc<dyn DesktopIntegration>,
    file_searcher: FileSearcher,
    current_searcher: Option<Box<dyn Searcher>>,
    // This type performs dumb id generation, but no checks. The reason is that checks must be performed
    // by the App type (e.g. display or not the entries sent from a search), so it's cleaner to perform
    // all of them there.
    //
    current_search_id: u32,
}

impl SearchManager {
    pub fn new(config: Config) -> Result<Self, String> {
        let desktop: Arc<dyn DesktopIntegration> = Arc::new(NativeDesktopIntegration);
        let file_searcher = FileSearcher::new(config, Arc::clone(&desktop))?;

        Ok(Self {
            desktop,
            file_searcher,
            current_searcher: None,
            current_search_id: 0,
        })
    }

    pub fn with_dependencies(
        config: Config,
        desktop: Arc<dyn DesktopIntegration>,
        home_dir: PathBuf,
    ) -> Result<Self, String> {
        let file_searcher = FileSearcher::with_home(config, Arc::clone(&desktop), home_dir)?;

        Ok(Self {
            desktop,
            file_searcher,
            current_searcher: None,
            current_search_id: 0,
        })
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
        let searchers: Vec<Box<dyn Searcher>> = vec![
            Box::new(EmojiSearcher::new(Arc::clone(&self.desktop))),
            Box::new(self.file_searcher.new_search()),
        ];

        searchers
            .into_iter()
            .find(|searcher| searcher.handles(pattern))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::Path,
        sync::atomic::{AtomicBool, Ordering},
    };

    use crate::search::search_result_entry::SearchResultEntry;

    use super::*;

    struct NoopDesktop;

    impl DesktopIntegration for NoopDesktop {
        fn copy_text(&self, _text: String) -> Result<(), String> {
            Ok(())
        }

        fn open_path(&self, _path: &Path) -> Result<(), String> {
            Ok(())
        }
    }

    struct NullSink;

    impl SearchResultSink for NullSink {
        fn send(&self, _entries: Vec<SearchResultEntry>) {}
    }

    struct StopRecordingSearcher {
        stopped: Arc<AtomicBool>,
    }

    impl Searcher for StopRecordingSearcher {
        fn handles(&self, _pattern: &str) -> bool {
            true
        }

        fn search(&mut self, _pattern: String, _sink: Arc<dyn SearchResultSink>, _search_id: u32) {}

        fn execute(&self, _value: String) -> Result<ExecutionAction, String> {
            Ok(ExecutionAction::Ignore)
        }

        fn stop(&mut self) {
            self.stopped.store(true, Ordering::Release);
        }
    }

    #[test]
    fn starting_a_new_search_stops_the_previous_searcher() {
        let mut manager = SearchManager::with_dependencies(
            Config {
                search_paths: Vec::new(),
                skip_paths: Vec::new(),
            },
            Arc::new(NoopDesktop),
            PathBuf::from("/unused"),
        )
        .unwrap();
        let stopped = Arc::new(AtomicBool::new(false));
        manager.current_searcher = Some(Box::new(StopRecordingSearcher {
            stopped: Arc::clone(&stopped),
        }));

        manager.search("x".to_string(), Arc::new(NullSink));

        assert!(stopped.load(Ordering::Acquire));
    }

    #[test]
    fn native_manager_construction_does_not_require_desktop_access() {
        let manager = SearchManager::new(Config {
            search_paths: Vec::new(),
            skip_paths: Vec::new(),
        });

        assert!(manager.is_ok());
    }
}
