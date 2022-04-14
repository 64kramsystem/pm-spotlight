#[derive(Clone)]
pub enum MessageEvent {
    UpdateList(String),
    SelectListEntry(String),
}
