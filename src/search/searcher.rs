use super::search_result_entry::SearchResultEntry;

pub trait Searcher {
    fn handles(&self, pattern: &str) -> bool;
    fn search(&mut self, pattern: String) -> Vec<SearchResultEntry>;
    fn execute(&self, entry: String);
}
