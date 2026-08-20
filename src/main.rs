mod gui {
    pub mod message_event;
    pub mod pm_spotlight_app;
}

mod search {
    pub mod emoji_searcher;
    pub mod file_searcher;
    pub mod search_manager;
    pub mod search_result_entry;
    pub mod searcher;
}

mod helpers {
    pub mod clipboard_management;
    pub mod filenames;
}

mod config {
    pub mod config_manager;
}

use config::config_manager::ConfigManager;
use gui::pm_spotlight_app::PMSpotlightApp;
use search::search_manager::SearchManager;
use std::process::ExitCode;

fn main() -> ExitCode {
    let config = match ConfigManager::load_configuration() {
        Ok(config) => config,
        Err(error) => {
            report_startup_error(&error);
            return ExitCode::FAILURE;
        }
    };
    let search_manager = SearchManager::new(config);
    PMSpotlightApp::build(search_manager).run();
    ExitCode::SUCCESS
}

fn report_startup_error(error: &str) {
    let message = format!("Poor Man's Spotlight could not start:\n\n{error}");

    eprintln!("{message}");

    if std::env::var_os("DISPLAY").is_some_and(|display| !display.is_empty()) {
        let _app = fltk::app::App::default();
        fltk::dialog::alert_default(&message);
    }
}
