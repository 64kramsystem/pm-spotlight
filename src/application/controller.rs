use std::sync::Arc;

use crate::search::{
    search_manager::SearchManager,
    search_result_entry::SearchResultEntry,
    searcher::{ExecutionAction, SearchResultSink},
};

/// Coordinates search state and execution independently of the GUI toolkit.
pub struct AppController {
    search_manager: SearchManager,
    current_search_id: u32,
}

impl AppController {
    pub fn new(search_manager: SearchManager) -> Self {
        Self {
            search_manager,
            current_search_id: 0,
        }
    }

    pub fn start_search(&mut self, pattern: String, sink: Arc<dyn SearchResultSink>) {
        self.current_search_id = self.search_manager.search(pattern, sink);
    }

    pub fn filter_current_entries(
        &self,
        entries: Vec<SearchResultEntry>,
    ) -> Vec<SearchResultEntry> {
        entries
            .into_iter()
            .filter(|entry| entry.search_id == self.current_search_id)
            .collect()
    }

    pub fn execute_entry(
        &mut self,
        entry: &SearchResultEntry,
        alternate: bool,
    ) -> Result<ExecutionAction, String> {
        if entry.search_id != self.current_search_id || !entry.valid {
            return Ok(ExecutionAction::Ignore);
        }

        let value = entry.value.clone().unwrap_or_else(|| entry.label.clone());
        if alternate {
            self.search_manager.alt_execute(value)
        } else {
            self.search_manager.execute(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        sync::Mutex,
    };

    use crate::{config::config_manager::Config, helpers::desktop_integration::DesktopIntegration};

    use super::*;

    #[derive(Default)]
    struct RecordingDesktop {
        copied: Mutex<Vec<String>>,
        opened: Mutex<Vec<PathBuf>>,
    }

    impl DesktopIntegration for RecordingDesktop {
        fn copy_text(&self, text: String) -> Result<(), String> {
            self.copied.lock().unwrap().push(text);
            Ok(())
        }

        fn open_path(&self, path: &Path) -> Result<(), String> {
            self.opened.lock().unwrap().push(path.to_path_buf());
            Ok(())
        }
    }

    struct NullSink;

    impl SearchResultSink for NullSink {
        fn send(&self, _entries: Vec<SearchResultEntry>) {}
    }

    fn controller(desktop: Arc<dyn DesktopIntegration>) -> AppController {
        let manager = SearchManager::with_dependencies(
            Config {
                search_paths: Vec::new(),
                skip_paths: Vec::new(),
            },
            desktop,
            std::env::temp_dir(),
        )
        .unwrap();
        AppController::new(manager)
    }

    fn entry(search_id: u32, valid: bool, value: Option<&str>) -> SearchResultEntry {
        SearchResultEntry::new(
            None,
            "fallback label".to_string(),
            value.map(str::to_string),
            search_id,
            valid,
        )
    }

    #[test]
    fn only_results_from_the_latest_search_reach_the_view() {
        let mut controller = controller(Arc::new(RecordingDesktop::default()));
        controller.start_search(String::new(), Arc::new(NullSink));

        let filtered = controller.filter_current_entries(vec![
            entry(0, true, Some("stale")),
            entry(1, true, Some("current")),
            entry(2, true, Some("future")),
        ]);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].value.as_deref(), Some("current"));
    }

    #[test]
    fn stale_and_invalid_entries_are_never_executed() {
        let desktop = Arc::new(RecordingDesktop::default());
        let mut controller = controller(desktop.clone());
        controller.start_search("file".to_string(), Arc::new(NullSink));

        assert_eq!(
            controller.execute_entry(&entry(0, true, Some("/stale")), false),
            Ok(ExecutionAction::Ignore)
        );
        assert_eq!(
            controller.execute_entry(&entry(1, false, Some("/invalid")), false),
            Ok(ExecutionAction::Ignore)
        );
        assert!(desktop.opened.lock().unwrap().is_empty());
    }

    #[test]
    fn execution_uses_the_entry_value_and_routes_through_the_active_searcher() {
        let desktop = Arc::new(RecordingDesktop::default());
        let mut controller = controller(desktop.clone());
        controller.start_search("file".to_string(), Arc::new(NullSink));

        assert_eq!(
            controller
                .execute_entry(&entry(1, true, Some("/selected/file")), false)
                .unwrap(),
            ExecutionAction::ExitApplication
        );
        assert_eq!(
            desktop.opened.lock().unwrap().as_slice(),
            [PathBuf::from("/selected/file")]
        );
    }

    #[test]
    fn execution_falls_back_to_the_label_when_an_entry_has_no_value() {
        let desktop = Arc::new(RecordingDesktop::default());
        let mut controller = controller(desktop.clone());
        controller.start_search(":".to_string(), Arc::new(NullSink));

        assert_eq!(
            controller.execute_entry(&entry(1, true, None), false),
            Ok(ExecutionAction::ExitApplication)
        );
        assert_eq!(
            desktop.copied.lock().unwrap().as_slice(),
            ["fallback label"]
        );
    }

    #[test]
    fn alternate_execution_is_routed_through_the_active_searcher() {
        let desktop = Arc::new(RecordingDesktop::default());
        let mut controller = controller(desktop.clone());
        controller.start_search("file".to_string(), Arc::new(NullSink));
        let path = std::env::temp_dir().join(format!(
            "pm-spotlight-controller-test-{}",
            std::process::id()
        ));
        std::fs::write(&path, b"test").unwrap();
        let canonical_path = std::fs::canonicalize(&path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let result = controller.execute_entry(&entry(1, true, Some(path.to_str().unwrap())), true);

        let _ = std::fs::remove_file(&path);
        assert_eq!(result, Ok(ExecutionAction::ExitApplication));
        assert_eq!(desktop.copied.lock().unwrap().as_slice(), [canonical_path]);
    }
}
