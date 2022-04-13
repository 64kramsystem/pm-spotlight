mod gui {
    pub mod app_builder;
    pub mod user_event;
    pub mod user_event_handler;
}

mod search {
    pub mod emoji_searcher;
    pub mod searcher;
}

use gui::app_builder::AppBuilder;
use gui::user_event_handler::UserEventHandler;
use search::emoji_searcher::EmojiSearcher;
use search::searcher::Searcher;

fn main() {
    let (app, mut browser, receiver) = AppBuilder::build();
    let user_event_handler = UserEventHandler::new();
    let searchers: Vec<Box<dyn Searcher>> = vec![Box::new(EmojiSearcher::new())];

    while app.wait() {
        if let Some(event) = receiver.recv() {
            user_event_handler.handle_event(event, &searchers, &mut browser);
        }
    }
}
