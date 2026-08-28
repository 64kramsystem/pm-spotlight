use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use pm_spotlight::{
    config::config_manager::Config,
    helpers::desktop_integration::DesktopIntegration,
    search::{
        search_manager::SearchManager,
        search_result_entry::SearchResultEntry,
        searcher::{ExecutionAction, SearchResultSink},
    },
};

static NEXT_TEMP_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

struct TempDirectory {
    path: PathBuf,
}

impl TempDirectory {
    fn new() -> Self {
        let sequence = NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "pm-spotlight-tests-{}-{timestamp}-{sequence}",
            std::process::id()
        ));

        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn create_file(&self, relative_path: &str) -> PathBuf {
        let path = self.path.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"test").unwrap();
        path
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Default)]
struct RecordingDesktop {
    copied_text: Mutex<Vec<String>>,
    opened_paths: Mutex<Vec<PathBuf>>,
    copy_error: Mutex<Option<String>>,
    open_error: Mutex<Option<String>>,
}

impl DesktopIntegration for RecordingDesktop {
    fn copy_text(&self, text: String) -> Result<(), String> {
        if let Some(error) = self.copy_error.lock().unwrap().clone() {
            return Err(error);
        }

        self.copied_text.lock().unwrap().push(text);
        Ok(())
    }

    fn open_path(&self, path: &Path) -> Result<(), String> {
        if let Some(error) = self.open_error.lock().unwrap().clone() {
            return Err(error);
        }

        self.opened_paths.lock().unwrap().push(path.to_path_buf());
        Ok(())
    }
}

struct CollectingSink {
    batches: Mutex<Vec<Vec<SearchResultEntry>>>,
    received: Condvar,
    send_delay: Duration,
}

impl Default for CollectingSink {
    fn default() -> Self {
        Self {
            batches: Mutex::new(Vec::new()),
            received: Condvar::new(),
            send_delay: Duration::ZERO,
        }
    }
}

impl CollectingSink {
    fn with_send_delay(send_delay: Duration) -> Self {
        Self {
            send_delay,
            ..Self::default()
        }
    }

    fn wait_for_batches(
        &self,
        expected_count: usize,
        timeout: Duration,
    ) -> Vec<Vec<SearchResultEntry>> {
        let batches = self.batches.lock().unwrap();
        let (batches, wait_result) = self
            .received
            .wait_timeout_while(batches, timeout, |batches| batches.len() < expected_count)
            .unwrap();

        assert!(
            !wait_result.timed_out(),
            "timed out waiting for search results"
        );
        batches.clone()
    }
}

impl SearchResultSink for CollectingSink {
    fn send(&self, entries: Vec<SearchResultEntry>) {
        std::thread::sleep(self.send_delay);
        self.batches.lock().unwrap().push(entries);
        self.received.notify_all();
    }
}

fn config(search_paths: &[&str], skip_paths: &[&str]) -> Config {
    Config {
        search_paths: search_paths
            .iter()
            .map(|path| (*path).to_string())
            .collect(),
        skip_paths: skip_paths.iter().map(|path| (*path).to_string()).collect(),
    }
}

#[test]
fn emoji_search_routes_execution_to_the_clipboard() {
    let home = TempDirectory::new();
    let desktop = Arc::new(RecordingDesktop::default());
    let sink = Arc::new(CollectingSink::default());
    let mut manager =
        SearchManager::with_dependencies(config(&[], &[]), desktop.clone(), home.path.clone());

    let search_id = manager.search(":".to_string(), sink.clone());

    assert_eq!(search_id, 1);
    assert!(sink.batches.lock().unwrap().is_empty());

    assert_eq!(
        manager.execute("👍".to_string()).unwrap(),
        ExecutionAction::ExitApplication
    );
    assert_eq!(desktop.copied_text.lock().unwrap().as_slice(), ["👍"]);
}

#[test]
fn file_search_honors_home_depth_hidden_and_skip_rules() {
    let home = TempDirectory::new();
    let visible = home.create_file("documents/visible_target.txt");
    let nested = home.create_file("documents/nested/nested_target.txt");
    home.create_file("documents/nested/deeper/too_deep_target.txt");
    home.create_file("documents/.hidden_target.txt");
    home.create_file("documents/ignored/ignored_target.txt");

    let desktop = Arc::new(RecordingDesktop::default());
    let sink = Arc::new(CollectingSink::default());
    let mut manager = SearchManager::with_dependencies(
        config(&["documents{2}"], &["documents/ignored"]),
        desktop,
        home.path.clone(),
    );

    let search_id = manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches.len(), 1);
    let mut values = batches[0]
        .iter()
        .map(|entry| {
            assert_eq!(entry.search_id, search_id);
            assert!(entry.valid);
            entry.value.clone().unwrap()
        })
        .collect::<Vec<_>>();
    values.sort();

    let mut expected = vec![
        visible.to_str().unwrap().to_string(),
        nested.to_str().unwrap().to_string(),
    ];
    expected.sort();
    assert_eq!(values, expected);
}

#[test]
fn file_execution_uses_the_desktop_integration_and_reports_failures() {
    let home = TempDirectory::new();
    let file = home.create_file("documents/target.txt");
    let desktop = Arc::new(RecordingDesktop::default());
    let sink = Arc::new(CollectingSink::default());
    let mut manager = SearchManager::with_dependencies(
        config(&["documents"], &[]),
        desktop.clone(),
        home.path.clone(),
    );
    manager.search("target".to_string(), sink.clone());
    sink.wait_for_batches(1, Duration::from_secs(2));

    assert_eq!(
        manager.execute(file.to_str().unwrap().to_string()).unwrap(),
        ExecutionAction::ExitApplication
    );
    assert_eq!(
        desktop.opened_paths.lock().unwrap().as_slice(),
        std::slice::from_ref(&file)
    );

    assert_eq!(
        manager
            .alt_execute(file.to_str().unwrap().to_string())
            .unwrap(),
        ExecutionAction::ExitApplication
    );
    assert_eq!(
        desktop.copied_text.lock().unwrap().as_slice(),
        [fs::canonicalize(&file).unwrap().to_str().unwrap()]
    );

    *desktop.open_error.lock().unwrap() = Some("desktop unavailable".to_string());
    let error = manager
        .execute(file.to_str().unwrap().to_string())
        .unwrap_err();
    assert!(error.contains("Could not open"));
    assert!(error.contains("desktop unavailable"));
}

#[test]
fn invalid_file_patterns_return_a_non_executable_message() {
    let home = TempDirectory::new();
    let sink = Arc::new(CollectingSink::default());
    let mut manager = SearchManager::with_dependencies(
        config(&[], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    let search_id = manager.search("bad;pattern".to_string(), sink.clone());

    let batches = sink.batches.lock().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 1);
    let entry = &batches[0][0];
    assert_eq!(entry.search_id, search_id);
    assert!(!entry.valid);
    assert!(entry.label.contains("Only alphanum"));
}

#[test]
fn every_search_gets_a_new_identifier_even_without_results() {
    let home = TempDirectory::new();
    let sink = Arc::new(CollectingSink::default());
    let mut manager = SearchManager::with_dependencies(
        config(&[], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    assert_eq!(manager.search(String::new(), sink.clone()), 1);
    assert_eq!(manager.search("x".to_string(), sink.clone()), 2);
    assert_eq!(manager.search(":not-present".to_string(), sink.clone()), 3);

    let batches = sink.batches.lock().unwrap();
    assert_eq!(batches.len(), 1);
    assert!(batches[0].is_empty());
}

#[test]
fn filesystem_search_returns_before_results_are_delivered() {
    let home = TempDirectory::new();
    home.create_file("documents/target.txt");
    let sink = Arc::new(CollectingSink::with_send_delay(Duration::from_millis(750)));
    let mut manager = SearchManager::with_dependencies(
        config(&["documents"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    let started = Instant::now();
    manager.search("target".to_string(), sink.clone());

    assert!(
        started.elapsed() < Duration::from_millis(500),
        "filesystem search waited for result delivery"
    );
    sink.wait_for_batches(1, Duration::from_secs(2));
}

#[test]
fn filesystem_worker_failures_are_reported_as_invalid_results() {
    let home = TempDirectory::new();
    home.create_file("documents/target.txt");

    let sink = Arc::new(CollectingSink::default());
    let mut manager = SearchManager::with_dependencies(
        config(&["documents", "documents"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    let search_id = manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 1);
    assert_eq!(batches[0][0].search_id, search_id);
    assert!(!batches[0][0].valid);
    assert!(batches[0][0]
        .label
        .contains("Filesystem search failed unexpectedly"));
}
