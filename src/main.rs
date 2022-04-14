mod gui {
    pub mod app_builder;
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

use gui::app_builder::AppBuilder;
use gui::user_event_handler::UserEventHandler;
use search::searchers_provider::SearchersProvider;

fn main() {
    let (app, browser, mut input, receiver) = AppBuilder::build();
    let mut user_event_handler = UserEventHandler::new();
    let searchers_provider = SearchersProvider::new();

    while app.wait() {
        if let Some(event) = receiver.recv() {
            user_event_handler.handle_event(event, &searchers_provider, &browser, &mut input);
        }
    }
}
