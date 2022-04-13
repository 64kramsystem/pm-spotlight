use fltk::{app::set_focus, browser::HoldBrowser, prelude::*};

use super::user_event::UserEvent::{self, *};
use crate::search::searchers_provider::SearchersProvider;

pub struct UserEventHandler {}

impl UserEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_event(
        &self,
        event: UserEvent,
        searchers_provider: &SearchersProvider,
        browser: &mut HoldBrowser,
    ) {
        match event {
            UpdateList(pattern) => {
                browser.clear();

                let searcher = searchers_provider.find_provider(&pattern);

                if let Some(searcher) = searcher {
                    let search_result = searcher.search(&pattern);

                    for entry in search_result {
                        browser.add(&entry);
                    }
                }
            }
            FocusOnList => {
                if browser.size() > 0 {
                    set_focus(browser);
                    browser.select(1);
                }
            }
            SelectListEntry(text) => {
                println!("selection: {}", text);
            }
        }
    }
}
