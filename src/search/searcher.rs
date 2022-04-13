pub trait Searcher {
    fn handles(&self, pattern: &str) -> bool;
    fn search(&self, pattern: &str) -> Option<Vec<String>>;
}
