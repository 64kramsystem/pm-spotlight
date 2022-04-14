#[derive(Clone)]
pub enum MessageEvent {
    UpdateList(String),
    ExecuteListEntry(String),
}
