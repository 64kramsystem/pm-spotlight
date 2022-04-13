use crate::user_event::UserEvent::{self, *};
use fltk::{app::set_focus, browser::HoldBrowser, prelude::*};

const ENTRIES_A: [&str; 3] = ["A:First", "A:Second", "A:Third"];
const ENTRIES_B: [&str; 3] = ["B:First", "B:Second", "B:Third"];

pub struct UserEventHandler {}

impl UserEventHandler {
    pub fn new() -> Self {
        Self {}
    }

    pub fn handle_event(&self, event: UserEvent, browser: &mut HoldBrowser) {
        match event {
            UpdateList(pattern) => {
                browser.clear();

                match pattern.as_str() {
                    "a" => {
                        for entry in ENTRIES_A {
                            browser.add(entry);
                        }
                    }
                    "b" => {
                        for entry in ENTRIES_B {
                            browser.add(entry);
                        }
                    }
                    _ => {
                        let entry = format!("<none: {}>", pattern);
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
