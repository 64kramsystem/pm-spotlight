#[derive(Clone)]
pub enum UserEvent {
    UpdateList(String),
    FocusOnList,
    SelectListEntry(String),
    Reset,
}
