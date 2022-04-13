pub trait Searcher {
    fn search(&self, pattern: &str) -> Option<Vec<String>>;
}
