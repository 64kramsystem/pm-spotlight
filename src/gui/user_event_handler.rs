use fltk::{app::set_focus, browser::HoldBrowser, input::Input, prelude::*};

use super::user_event::UserEvent::{self, *};
use crate::search::{searcher::Searcher, searchers_provider::SearchersProvider};

pub struct UserEventHandler {
    current_searcher: Option<Box<dyn Searcher>>,
}

impl UserEventHandler {
    pub fn new() -> Self {
        Self {
            current_searcher: None,
        }
    }

    pub fn handle_event(
        &mut self,
        event: UserEvent,
        searchers_provider: &SearchersProvider,
        browser: &mut HoldBrowser,
        input: &mut Input,
    ) {
        match event {
            UpdateList(pattern) => {
                browser.clear();

                self.current_searcher = searchers_provider.find_provider(&pattern);

                if let Some(searcher) = &mut self.current_searcher {
                    let search_result = searcher.search(&pattern);

                    for (entry_text, entry_data) in search_result {
                        if let Some(entry_data) = entry_data {
                            browser.add_with_data(&entry_text, entry_data);
                        } else {
                            browser.add(&entry_text);
                        }
                    }
                }
            }
            FocusOnList => {
                if browser.size() > 0 {
                    set_focus(browser);
                    browser.select(1);
                }
            }
            SelectListEntry(entry) => {
                if let Some(searcher) = &self.current_searcher {
                    searcher.execute(entry);
                }
            }
            Reset => {
                input.set_value("");
                set_focus(input);
                browser.clear();
            }
        }
    }
}
