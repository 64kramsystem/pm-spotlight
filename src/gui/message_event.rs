use fltk::app::Sender;

use crate::search::{search_result_entry::SearchResultEntry, searcher::SearchResultSink};

#[derive(Clone)]
pub enum MessageEvent {
    StartSearch(String),
    UpdateList(Vec<SearchResultEntry>),
    FocusOnBrowser,
    // False: normal; true: alternate
    ExecuteEntry(bool),
}

impl SearchResultSink for Sender<MessageEvent> {
    fn send(&self, entries: Vec<SearchResultEntry>) {
        Sender::send(self, MessageEvent::UpdateList(entries));
    }
}
