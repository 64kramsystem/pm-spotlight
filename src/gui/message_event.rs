use crate::search::search_result_entry::SearchResultEntry;

#[derive(Clone)]
pub enum MessageEvent {
    StartSearch(String),
    UpdateList(Vec<SearchResultEntry>),
    FocusOnBrowser,
    // False: normal; true: alternate
    ExecuteEntry(bool),
}
