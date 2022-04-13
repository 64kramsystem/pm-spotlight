use fltk::{app::set_focus, browser::HoldBrowser, prelude::*};

use super::user_event::UserEvent::{self, *};
use crate::search::searcher::Searcher;

pub struct UserEventHandler {}

impl UserEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_event(
        &self,
        event: UserEvent,
        searchers: &Vec<Box<dyn Searcher>>,
        browser: &mut HoldBrowser,
    ) {
        match event {
            UpdateList(pattern) => {
                browser.clear();

                let search_result = searchers
                    .iter()
                    .find_map(|searcher| searcher.search(&pattern));

                if let Some(entries_list) = search_result {
                    for entry in &entries_list {
                        browser.add(entry);
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
