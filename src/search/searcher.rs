use fltk::prelude::ImageExt;

pub trait Searcher {
    fn handles(&self, pattern: &str) -> bool;
    // Tuple: (icon, text, data).
    //
    fn search(&mut self, pattern: &str)
        -> Vec<(Option<Box<dyn ImageExt>>, String, Option<String>)>;
    fn execute(&self, entry: String);
}
