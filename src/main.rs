mod gui {
    pub mod app_builder;
    pub mod user_event;
    pub mod user_event_handler;
}

use gui::app_builder::AppBuilder;
use gui::user_event_handler::UserEventHandler;

fn main() {
    let (app, mut browser, receiver) = AppBuilder::build();
    let user_event_handler = UserEventHandler::new();

    while app.wait() {
        if let Some(event) = receiver.recv() {
            user_event_handler.handle_event(event, &mut browser);
        }
    }
}
