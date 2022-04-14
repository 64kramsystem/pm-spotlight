pub trait Searcher {
    fn handles(&self, pattern: &str) -> bool;
    // Currently, a flat list of entries is returned, instead of a map, which would allow a view pattern.
    // The result is that certain functionalities can't be implemented, e.g. a file search that doesn't
    // display the whole path. This will be fixed once the list widget will implement such pattern.
    fn search(&mut self, pattern: &str) -> Vec<(String, Option<String>)>;
    fn execute(&self, entry: String);
}
