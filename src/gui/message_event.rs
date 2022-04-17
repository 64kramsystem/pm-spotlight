use crate::search::search_result_entry::SearchResultEntry;

pub enum MessageEvent {
    Search(String),
    UpdateList(Vec<SearchResultEntry>),
    FocusOnList,
    ExecuteListEntry(SearchResultEntry),
}
