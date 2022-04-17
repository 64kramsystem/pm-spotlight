use fltk::{
    app::{self, set_focus, App, Receiver, Sender},
    browser::HoldBrowser,
    enums::{CallbackTrigger, Event, Key},
    group::Pack,
    input::Input,
    prelude::*,
    window::Window,
};

use crate::search::{search_manager::SearchManager, search_result_entry::SearchResultEntry};

use super::message_event::MessageEvent::{self, *};

const WINDOW_TITLE: &str = "Poor Man's Spotlight!";

const WINDOW_WIDTH: i32 = 350;
const WINDOW_HEIGHT: i32 = 500;

const BROWSER_TEXT_SIZE: i32 = 15; // default: 14

pub struct PMSpotlightApp {
    search_manager: SearchManager,
    current_search_id: u32,
    app: App,
    sender: Sender<MessageEvent>,
    receiver: Receiver<MessageEvent>,
    browser: HoldBrowser,
    input: Input,
}

impl PMSpotlightApp {
    pub fn build(search_manager: SearchManager) -> Self {
        let app = App::default();
        let mut window = Window::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .with_label(WINDOW_TITLE);
        let pack = Pack::default().size_of(&window);

        let (sender, receiver) = app::channel();

        let mut input = Input::default().with_size(0, 25);
        let mut browser = HoldBrowser::default_fill();

        browser.set_text_size(BROWSER_TEXT_SIZE);
        input.set_trigger(CallbackTrigger::Changed);

        Self::callback_start_search(&mut input, sender.clone());
        Self::fltk_event_list_execute_entry_and_focus_on_browser(&mut input, sender.clone());
        Self::fltk_event_execute_entry_from_browser(&mut browser, sender.clone());

        pack.end();
        window.end();
        window.show();

        Self {
            search_manager,
            current_search_id: 0,
            app,
            sender,
            receiver,
            browser,
            input,
        }
    }

    pub fn run(&mut self) {
        while self.app.wait() {
            if let Some(event) = self.receiver.recv() {
                match event {
                    StartSearch(pattern) => {
                        self.message_event_start_search(pattern);
                    }
                    UpdateList(entries) => {
                        self.message_event_update_list(entries);
                    }
                    FocusOnBrowser => {
                        self.message_event_focus_on_browser();
                    }
                    ExecuteEntry => {
                        self.message_event_execute_entry();
                    }
                }
            }
        }
    }

    /***************************************************************************
     * Callbacks
     ***************************************************************************/

    fn callback_start_search(input: &mut Input, sender: Sender<MessageEvent>) {
        input.set_callback(move |input| {
            let pattern = input.value();
            sender.send(StartSearch(pattern));
        });
    }

    /***************************************************************************
     * FLTK event handlers
     ***************************************************************************/

    // Can't use multiple handlers on the same widget.
    //
    fn fltk_event_list_execute_entry_and_focus_on_browser(
        input: &mut Input,
        sender: Sender<MessageEvent>,
    ) {
        input.handle(move |_input, event| {
            if event == Event::KeyDown && app::event_key() == Key::Enter {
                sender.send(ExecuteEntry);
                return true;
            } else if event == Event::KeyDown && app::event_key() == Key::Down {
                sender.send(FocusOnBrowser);
                return true;
            }

            false
        });
    }

    fn fltk_event_execute_entry_from_browser(
        browser: &mut HoldBrowser,
        sender: Sender<MessageEvent>,
    ) {
        // It seems that Enter-initiated callback is not supported for browsers.
        //
        browser.handle(move |_browser, event| {
            if event == Event::KeyDown && app::event_key() == Key::Enter {
                sender.send(ExecuteEntry);
                return true;
            }

            false
        });
    }

    /***************************************************************************
     * MessageEvent handlers
     ***************************************************************************/

    fn message_event_start_search(&mut self, pattern: String) {
        self.browser.clear();
        self.current_search_id = self.search_manager.search(pattern, self.sender.clone());
    }

    fn message_event_update_list(&mut self, entries: Vec<SearchResultEntry>) {
        for entry in entries {
            // Can check here or only on the single entry; doesn't matter.
            //
            if self.current_search_id == entry.search_id {
                // This is wasteful, but the browser wants to own the data. We could keep in #data just
                // the data strictly needed to perform the execute action, but it's an optimization that
                // doesn't matter, at least now.
                //
                let label = entry.label.clone();
                let icon = entry.icon.clone();

                self.browser.add_with_data(&label, entry);
                self.browser.set_icon(self.browser.size(), icon);
            }
        }
    }

    fn message_event_focus_on_browser(&mut self) {
        if self.browser.size() > 0 {
            set_focus(&self.browser);
            self.browser.select(1);
        }
    }

    fn message_event_execute_entry(&mut self) {
        let selected_line = if self.browser.value() > 0 {
            self.browser.value()
        } else if self.browser.size() >= 0 {
            1
        } else {
            return;
        };

        let entry: SearchResultEntry = unsafe { self.browser.data(selected_line) }.unwrap();

        if self.current_search_id == entry.search_id {
            self.search_manager
                .execute(entry.value.unwrap_or(entry.label));

            self.input.set_value("");
            set_focus(&self.input);
            self.browser.clear();
        }
    }
}
