use fltk::image::SharedImage;

#[derive(Clone)]
pub struct SearchResultEntry {
    pub icon: Option<SharedImage>,
    pub label: String,
    pub value: Option<String>,
}

impl SearchResultEntry {
    pub fn new(icon: Option<SharedImage>, text: String, value: Option<String>) -> Self {
        Self {
            icon,
            label: text,
            value,
        }
    }
}
