#[derive(Clone)]
pub enum UserEvent {
    UpdateList(String),
    SelectListEntry(String),
}
