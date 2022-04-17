use fltk::app::Sender;

use crate::gui::message_event::MessageEvent;

pub trait Searcher {
    fn handles(&self, pattern: &str) -> bool;
    // Can run in a separate thread or not, but in the latter case, the search **must** be so fast that
    // it's immediate from a user perspective.
    //
    fn search(&mut self, pattern: String, sender: Sender<MessageEvent>);
    fn execute(&self, value: String);
}
