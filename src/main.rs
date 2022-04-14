mod gui {
    pub mod pm_spotlight_app;
    pub mod user_event;
    pub mod user_event_handler;
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
use gui::user_event_handler::UserEventHandler;
use search::searchers_provider::SearchersProvider;

fn main() {
    let user_event_handler = UserEventHandler::new();
    let searchers_provider = SearchersProvider::new();

    PMSpotlightApp::build(user_event_handler, searchers_provider).run();
}
