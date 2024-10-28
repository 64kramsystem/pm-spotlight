#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod gui {
    pub mod message_event;
    pub mod pm_spotlight_app;
}

mod search {
    pub mod emoji_searcher;
    #[cfg(target_os = "linux")]
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

fn main() {
    let config = ConfigManager::load_configuration();
    let search_manager = SearchManager::new(config);
    PMSpotlightApp::build(search_manager).run();
}
