mod gui {
    pub mod message_event;
    pub mod pm_spotlight_app;
}

mod search {
    pub mod emoji_searcher;
    pub mod searcher;
    pub mod searchers_provider;
}

mod helpers {
    pub mod clipboard_management;
}

use gui::pm_spotlight_app::PMSpotlightApp;
use search::searchers_provider::SearchersProvider;

fn main() {
    let searchers_provider = SearchersProvider::new();
    PMSpotlightApp::build(searchers_provider).run();
}
