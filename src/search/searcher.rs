pub trait Searcher {
    fn handles(&self, pattern: &str) -> bool;
    fn search(&self, pattern: &str) -> Vec<String>;
    fn execute(&self, entry: String);
}
