use std::sync::Arc;

use super::search_result_entry::SearchResultEntry;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionAction {
    ExitApplication,
    Ignore,
}

pub trait SearchResultSink: Send + Sync {
    fn send(&self, entries: Vec<SearchResultEntry>);
}

pub trait Searcher {
    fn handles(&self, pattern: &str) -> bool;

    // The implementation must take care of not running on empty patterns. This is because "empty
    // pattern" is different from "empty string".
    //
    // Can run in a separate thread or not, but in the latter case, the search **must** be so fast that
    // it's immediate from a user perspective.
    //
    fn search(&mut self, pattern: String, sink: Arc<dyn SearchResultSink>, search_id: u32);

    fn execute(&self, value: String) -> Result<ExecutionAction, String>;

    // Alternate execute mode, activated by Shift+Enter; optional.
    //
    fn alt_execute(&self, _value: String) -> Result<ExecutionAction, String> {
        Ok(ExecutionAction::Ignore)
    }

    // Implemented only when there is a separate thread.
    //
    fn stop(&mut self) {}
}
