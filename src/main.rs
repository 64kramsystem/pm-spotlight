mod gui {
    pub mod message_event;
    pub mod pm_spotlight_app;
}

mod search {
    pub mod emoji_searcher;
    pub mod search_manager;
    pub mod search_result_entry;
    pub mod searcher;
}

mod helpers {
    pub mod clipboard_management;
}

mod config {
    pub mod config_manager;
}

use config::config_manager::ConfigManager;
use gui::pm_spotlight_app::PMSpotlightApp;
use search::search_manager::SearchManager;

fn main() {
    let _config = ConfigManager::load_configuration();
    let search_manager = SearchManager::new();
    PMSpotlightApp::build(search_manager).run();
}
