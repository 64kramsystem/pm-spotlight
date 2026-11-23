use fltk::image::SharedImage;

#[derive(Clone)]
pub struct SearchResultEntry {
    pub icon: Option<SharedImage>,
    pub label: String,
    pub value: Option<String>,
    // This is wasteful, as entries are sent in batch; additionally, the App current search id is enough,
    // however, it's more solid to perfom the check at individual entry level, since it's much more
    // solid, because operations doesn't require underlying assumptions.
    // For example, when an entry is executed, there's no need to check for a search_id, because new
    // searches clear the current browser list. However, this assumption introduces a dependency. By
    // storing the search id here, we don't need to care about how the app behaves.
    pub search_id: u32,
    // Invalid entries are not executed; they are used to convey messages to the user.
    pub valid: bool,
}

impl SearchResultEntry {
    pub fn new(
        icon: Option<SharedImage>,
        label: String,
        value: Option<String>,
        search_id: u32,
        valid: bool,
    ) -> Self {
        Self {
            icon,
            label,
            value,
            search_id,
            valid,
        }
    }
}
