use fltk::{
    app::{self, event_key_down, focus, set_focus, App},
    browser::HoldBrowser,
    enums::{CallbackTrigger, Key},
    group::Pack,
    input::Input,
    prelude::*,
    window::Window,
};

use UserEvent::*;

const WINDOW_WIDTH: i32 = 300;
const WINDOW_HEIGHT: i32 = 500;

const ENTRIES_A: [&str; 3] = ["A:First", "A:Second", "A:Third"];
const ENTRIES_B: [&str; 3] = ["B:First", "B:Second", "B:Third"];

#[derive(Clone)]
enum UserEvent {
    UpdateList(String),
    FocusOnList,
    SelectListEntry(String),
}

fn main() {
    let app = App::default();
    let mut window = Window::default().with_size(WINDOW_WIDTH, WINDOW_HEIGHT);
    let pack = Pack::default().size_of(&window);

    let (sender_i1, receiver) = app::channel();
    let sender_i2 = sender_i1.clone();
    let sender_b = sender_i1.clone();

    let mut input = Input::default().with_size(0, 25);

    input.set_trigger(CallbackTrigger::Changed);
    input.set_callback(move |input| {
        sender_i1.send(UpdateList(input.value()));
    });

    input.handle(move |input, _| {
        if event_key_down(Key::Down) {
            if let Some(focused) = focus() {
                if focused.is_same(input) {
                    sender_i2.send(FocusOnList);

                    return true;
                }
            }
        }

        false
    });

    let mut browser = HoldBrowser::default_fill();

    // It seems that Enter-initiated callback is not supported for browsers.
    //
    browser.handle(move |browser, _| {
        // WATCH OUT! This event is global (not for the browser only).
        //
        if event_key_down(Key::Enter) {
            if let Some(text) = browser.selected_text() {
                sender_b.send(SelectListEntry(text));
            } else if let Some(text) = browser.text(1) {
                sender_b.send(SelectListEntry(text));
            }

            return true;
        }

        false
    });

    pack.end();
    window.end();
    window.show();

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
