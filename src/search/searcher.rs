use fltk::image::PngImage;

pub trait Searcher {
    fn handles(&self, pattern: &str) -> bool;
    // Tuple: (icon, text, data).
    //
    fn search(&mut self, pattern: &str) -> Vec<(Option<PngImage>, String, Option<String>)>;
    fn execute(&self, entry: String);
}
