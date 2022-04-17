mod gui {
    pub mod message_event;
    pub mod pm_spotlight_app;
}

mod search {
    pub mod emoji_searcher;
    pub mod search_result_entry;
    pub mod searcher;
    pub mod searchers_provider;
}

mod helpers {
    pub mod clipboard_management;
}

mod config {
    pub mod config_manager;
}

use config::config_manager::ConfigManager;
use gui::pm_spotlight_app::PMSpotlightApp;
use search::searchers_provider::SearchersProvider;

fn main() {
    let _config = ConfigManager::load_configuration();
    let searchers_provider = SearchersProvider::new();
    PMSpotlightApp::build(searchers_provider).run();
}
