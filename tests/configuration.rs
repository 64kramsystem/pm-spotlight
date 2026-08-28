use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use pm_spotlight::config::config_manager::{Config, ConfigManager};

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
            "pm-spotlight-config-tests-{}-{timestamp}-{sequence}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn config_path(&self) -> PathBuf {
        self.path.join(".pm-spotlight")
    }

    fn write_config(&self, contents: &str) {
        fs::write(self.config_path(), contents).unwrap();
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn load(home: &Path) -> Result<Config, String> {
    ConfigManager::load_configuration_from(home)
}

#[test]
fn missing_configuration_is_created_with_an_actionable_error() {
    let home = TempDirectory::new();

    let error = load(&home.path).unwrap_err();

    assert!(error.contains("Created"));
    assert!(error.contains("search_paths"));
    assert_eq!(
        fs::read_to_string(home.config_path()).unwrap(),
        "search_paths = []\nskip_paths = []\n"
    );
}

#[test]
fn valid_configuration_is_loaded_verbatim() {
    let home = TempDirectory::new();
    home.write_config(
        "search_paths = [\"Documents{3}\", \"/shared\"]\nskip_paths = [\"target\"]\n",
    );

    assert_eq!(
        load(&home.path).unwrap(),
        Config {
            search_paths: vec!["Documents{3}".to_string(), "/shared".to_string()],
            skip_paths: vec!["target".to_string()],
        }
    );
}

#[test]
fn empty_search_paths_are_rejected() {
    let home = TempDirectory::new();
    home.write_config("search_paths = []\nskip_paths = []\n");

    let error = load(&home.path).unwrap_err();

    assert!(error.contains("No search paths are configured"));
    assert!(error.contains("search_paths"));
}

#[test]
fn malformed_toml_is_reported_with_the_configuration_path() {
    let home = TempDirectory::new();
    home.write_config("search_paths = [\n");

    let error = load(&home.path).unwrap_err();

    assert!(error.contains("Could not parse"));
    assert!(error.contains(home.config_path().to_str().unwrap()));
}

#[test]
fn missing_required_fields_are_configuration_errors() {
    let home = TempDirectory::new();
    home.write_config("search_paths = [\"Documents\"]\n");

    let error = load(&home.path).unwrap_err();

    assert!(error.contains("Could not parse"));
    assert!(error.contains("skip_paths"));
}

#[test]
fn unreadable_configuration_locations_are_reported() {
    let home = TempDirectory::new();
    fs::create_dir(home.config_path()).unwrap();

    let error = load(&home.path).unwrap_err();

    assert!(error.contains("Could not read"));
    assert!(error.contains(home.config_path().to_str().unwrap()));
}

#[test]
fn configuration_creation_failures_are_reported() {
    let parent = TempDirectory::new();
    let missing_home = parent.path.join("missing-home");

    let error = load(&missing_home).unwrap_err();

    assert!(error.contains("Could not create"));
    assert!(error.contains(".pm-spotlight"));
}
