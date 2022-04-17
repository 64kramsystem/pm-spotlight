use fltk::image::SharedImage;

#[derive(Clone)]
pub struct SearchResultEntry {
    pub icon: Option<SharedImage>,
    pub text: String,
    pub data: Option<String>,
}

impl SearchResultEntry {
    pub fn new(icon: Option<SharedImage>, text: String, data: Option<String>) -> Self {
        Self { icon, text, data }
    }
}
