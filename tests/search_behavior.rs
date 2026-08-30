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

fn search_manager(
    config: Config,
    desktop: Arc<dyn DesktopIntegration>,
    home_dir: PathBuf,
) -> SearchManager {
    SearchManager::with_dependencies(config, desktop, home_dir).unwrap()
}

#[test]
fn emoji_search_routes_execution_to_the_clipboard() {
    let home = TempDirectory::new();
    let desktop = Arc::new(RecordingDesktop::default());
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(config(&[], &[]), desktop.clone(), home.path.clone());

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
fn execution_without_an_active_search_is_ignored() {
    let home = TempDirectory::new();
    let mut manager = search_manager(
        config(&[], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    assert_eq!(
        manager.execute("anything".to_string()).unwrap(),
        ExecutionAction::Ignore
    );
    assert_eq!(
        manager.alt_execute("anything".to_string()).unwrap(),
        ExecutionAction::Ignore
    );
}

#[test]
fn emoji_clipboard_failures_are_returned_and_alternate_execution_is_ignored() {
    let home = TempDirectory::new();
    let desktop = Arc::new(RecordingDesktop::default());
    *desktop.copy_error.lock().unwrap() = Some("clipboard unavailable".to_string());
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(config(&[], &[]), desktop, home.path.clone());
    manager.search(":".to_string(), sink);

    let error = manager.execute("👍".to_string()).unwrap_err();
    assert!(error.contains("Could not copy emoji"));
    assert!(error.contains("clipboard unavailable"));
    assert_eq!(
        manager.alt_execute("👍".to_string()).unwrap(),
        ExecutionAction::Ignore
    );
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
    let mut manager = search_manager(
        config(&["documents{2}"], &["documents/ignor*"]),
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
fn file_search_is_case_insensitive_and_supports_wildcards() {
    let home = TempDirectory::new();
    let alpha = home.create_file("documents/Alpha-report.TXT");
    let alphabet = home.create_file("documents/alphabet.txt");
    home.create_file("documents/beta.txt");
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["documents"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("ALPHA*TXT".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    let mut values = batches[0]
        .iter()
        .map(|entry| entry.value.clone().unwrap())
        .collect::<Vec<_>>();
    values.sort();
    let mut expected = vec![
        alpha.to_str().unwrap().to_string(),
        alphabet.to_str().unwrap().to_string(),
    ];
    expected.sort();
    assert_eq!(values, expected);
}

#[test]
fn file_search_orders_results_by_basename_match_closeness() {
    let home = TempDirectory::new();
    let exact = home.path.join("documents/mp3");
    let prefix = home.path.join("documents/mp3-tools");
    fs::create_dir_all(&exact).unwrap();
    fs::create_dir_all(&prefix).unwrap();
    let early_substring = home.create_file("documents/music/my-mp3");
    let later_substring = home.create_file("documents/music/song.mp3");
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["documents"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("mp3".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    let values = batches[0]
        .iter()
        .map(|entry| entry.value.as_deref().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(
        values,
        [
            exact.to_str().unwrap(),
            prefix.to_str().unwrap(),
            early_substring.to_str().unwrap(),
            later_substring.to_str().unwrap(),
        ]
    );
}

#[test]
fn periods_in_file_queries_are_matched_literally() {
    let home = TempDirectory::new();
    let expected = home.create_file("documents/report.txt");
    home.create_file("documents/reportXtxt");
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["documents"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("report.txt".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches[0].len(), 1);
    assert_eq!(batches[0][0].value.as_deref(), expected.to_str());
}

#[test]
fn absolute_and_missing_search_roots_are_handled() {
    let home = TempDirectory::new();
    let expected = home.create_file("absolute/target.txt");
    let absolute_root = home.path.join("absolute");
    let missing_root = home.path.join("not-mounted");
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(
            &[
                absolute_root.to_str().unwrap(),
                missing_root.to_str().unwrap(),
            ],
            &[],
        ),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches[0].len(), 1);
    assert_eq!(batches[0][0].value.as_deref(), expected.to_str());
}

#[test]
fn hidden_directories_are_not_traversed() {
    let home = TempDirectory::new();
    home.create_file("documents/.private/hidden_target.txt");
    let expected = home.create_file("documents/public/visible_target.txt");
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["documents"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches[0].len(), 1);
    assert_eq!(batches[0][0].value.as_deref(), expected.to_str());
}

#[test]
fn file_execution_uses_the_desktop_integration_and_reports_failures() {
    let home = TempDirectory::new();
    let file = home.create_file("documents/target.txt");
    let desktop = Arc::new(RecordingDesktop::default());
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
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

    *desktop.copy_error.lock().unwrap() = Some("clipboard unavailable".to_string());
    let error = manager
        .alt_execute(file.to_str().unwrap().to_string())
        .unwrap_err();
    assert!(error.contains("Could not copy the path"));
    assert!(error.contains("clipboard unavailable"));

    let missing = home.path.join("missing.txt");
    let error = manager
        .alt_execute(missing.to_str().unwrap().to_string())
        .unwrap_err();
    assert!(error.contains("Could not resolve"));
}

#[test]
fn invalid_file_patterns_return_a_non_executable_message() {
    let home = TempDirectory::new();
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
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
    let mut manager = search_manager(
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
    let mut manager = search_manager(
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
fn overlapping_search_roots_return_each_file_once() {
    let home = TempDirectory::new();
    let target = home.create_file("documents/nested/target.txt");

    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["documents", "documents/nested"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    let search_id = manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 1);
    assert_eq!(batches[0][0].search_id, search_id);
    assert!(batches[0][0].valid);
    assert_eq!(batches[0][0].value.as_deref(), target.to_str());
}

#[test]
fn skip_paths_treat_regex_metacharacters_as_literals() {
    let home = TempDirectory::new();
    home.create_file("documents/[archive]+(old)?/skipped_target.txt");
    let visible = home.create_file("documents/visible_target.txt");
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["documents"], &["documents/[archive]+(old)?"]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 1);
    assert_eq!(batches[0][0].value.as_deref(), visible.to_str());
}

#[test]
fn skip_paths_are_case_insensitive_and_expand_wildcards() {
    let home = TempDirectory::new();
    home.create_file("Documents/Archive-2025/skipped_target.txt");
    let visible = home.create_file("Documents/current/visible_target.txt");
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["Documents"], &["documents/archive-*"]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches[0].len(), 1);
    assert_eq!(batches[0][0].value.as_deref(), visible.to_str());
}

#[cfg(unix)]
#[test]
fn directory_symlinks_are_not_followed() {
    use std::os::unix::fs::symlink;

    let home = TempDirectory::new();
    home.create_file("outside/target.txt");
    fs::create_dir_all(home.path.join("documents")).unwrap();
    symlink(
        home.path.join("outside"),
        home.path.join("documents/linked-directory"),
    )
    .unwrap();
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["documents"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert!(batches[0].is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_filenames_are_skipped_without_failing_the_search() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let home = TempDirectory::new();
    let documents = home.path.join("documents");
    fs::create_dir(&documents).unwrap();
    fs::write(
        documents.join(OsString::from_vec(b"invalid_target_\xff".to_vec())),
        b"test",
    )
    .unwrap();
    let visible = home.create_file("documents/visible_target.txt");
    let sink = Arc::new(CollectingSink::default());
    let mut manager = search_manager(
        config(&["documents"], &[]),
        Arc::new(RecordingDesktop::default()),
        home.path.clone(),
    );

    manager.search("target".to_string(), sink.clone());

    let batches = sink.wait_for_batches(1, Duration::from_secs(2));
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].len(), 1);
    assert_eq!(batches[0][0].value.as_deref(), visible.to_str());
}

#[cfg(unix)]
#[test]
fn invalid_expanded_skip_paths_return_configuration_errors() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};

    let home_dir = PathBuf::from(OsString::from_vec(b"/tmp/pm-spotlight-\xff".to_vec()));
    let result = SearchManager::with_dependencies(
        config(&[], &["ignored"]),
        Arc::new(RecordingDesktop::default()),
        home_dir,
    );

    match result {
        Ok(_) => panic!("invalid expanded skip path was accepted"),
        Err(error) => assert!(error.contains("not valid UTF-8")),
    }
}
