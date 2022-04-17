use crate::search::search_result_entry::SearchResultEntry;

pub enum MessageEvent {
    StartSearch(String),
    UpdateList(Vec<SearchResultEntry>),
    FocusOnBrowser,
    ExecuteEntry(SearchResultEntry),
}
