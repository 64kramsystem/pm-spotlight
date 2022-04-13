mod app_builder;
mod user_event;
mod user_event_handler;

use app_builder::AppBuilder;
use user_event_handler::UserEventHandler;

fn main() {
    let (app, mut browser, receiver) = AppBuilder::build();
    let user_event_handler = UserEventHandler::new();

    while app.wait() {
        if let Some(event) = receiver.recv() {
            user_event_handler.handle_event(event, &mut browser);
        }
    }
}
