#[derive(Clone)]
pub enum MessageEvent {
    UpdateList(String),
    FocusOnList,
    ExecuteListEntry(String),
}
