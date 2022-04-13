mod app_builder;
mod user_event;

use fltk::{app::set_focus, browser::HoldBrowser, prelude::*};

use app_builder::AppBuilder;
use user_event::UserEvent::{self, *};

const ENTRIES_A: [&str; 3] = ["A:First", "A:Second", "A:Third"];
const ENTRIES_B: [&str; 3] = ["B:First", "B:Second", "B:Third"];

fn main() {
    let (app, mut browser, receiver) = AppBuilder::build();

    while app.wait() {
        if let Some(event) = receiver.recv() {
            handle_user_event(event, &mut browser);
        }
    }
}

fn handle_user_event(event: UserEvent, browser: &mut HoldBrowser) {
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
