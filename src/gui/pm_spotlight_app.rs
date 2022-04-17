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

        Self::callback_update_list(&mut input, sender.clone());
        Self::fltk_event_move_from_input_to_list(&mut input, sender.clone());
        Self::fltk_event_select_list_entry(&mut browser, sender.clone());

        pack.end();
        window.end();
        window.show();

        Self {
            search_manager,
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
                    Search(pattern) => {
                        self.message_event_search(pattern);
                    }
                    UpdateList(entries) => {
                        self.message_event_update_list(entries);
                    }
                    FocusOnList => {
                        self.message_event_focus_on_list();
                    }

                    ExecuteListEntry(entry) => {
                        self.message_event_execute_entry(entry);
                    }
                }
            }
        }
    }

    /***************************************************************************
     * Callbacks
     ***************************************************************************/

    fn callback_update_list(input: &mut Input, sender: Sender<MessageEvent>) {
        input.set_callback(move |input| {
            let pattern = input.value();
            sender.send(Search(pattern));
        });
    }

    /***************************************************************************
     * FLTK event handlers
     ***************************************************************************/

    fn fltk_event_move_from_input_to_list(input: &mut Input, sender: Sender<MessageEvent>) {
        input.handle(move |_input, event| {
            if event == Event::KeyDown && app::event_key() == Key::Down {
                sender.send(FocusOnList);
                return true;
            }

            false
        });
    }

    fn fltk_event_select_list_entry(browser: &mut HoldBrowser, sender: Sender<MessageEvent>) {
        // It seems that Enter-initiated callback is not supported for browsers.
        //
        browser.handle(move |browser, event| {
            // An alternative solution was to reset when tapping key up from the topmost Browser entry,
            // but this is not feasible with fltk(-rs), because:
            //
            // - the event is fired after the selection is changed
            // - the selection doesn't go above the first entry
            //
            if event == Event::KeyDown && app::event_key() == Key::Enter {
                let selected_line = if browser.value() > 0 {
                    browser.value()
                } else if browser.size() >= 0 {
                    1
                } else {
                    return true;
                };

                if let Some::<String>(text) = unsafe { browser.data(selected_line) } {
                    sender.send(ExecuteListEntry(text));
                } else if let Some(text) = browser.text(selected_line) {
                    sender.send(ExecuteListEntry(text));
                }

                return true;
            }

            false
        });
    }

    /***************************************************************************
     * MessageEvent handlers
     ***************************************************************************/

    fn message_event_search(&mut self, pattern: String) {
        self.browser.clear();
        self.search_manager.search(pattern, self.sender.clone());
    }

    fn message_event_update_list(&mut self, entries: Vec<SearchResultEntry>) {
        self.set_list_entries(entries);
    }

    fn message_event_focus_on_list(&mut self) {
        if self.browser.size() > 0 {
            set_focus(&self.browser);
            self.browser.select(1);
        }
    }

    fn message_event_execute_entry(&mut self, entry: String) {
        self.search_manager.execute(entry);

        self.input.set_value("");
        set_focus(&self.input);
        self.browser.clear();
    }

    /***************************************************************************
     * Helpers
     ***************************************************************************/

    fn set_list_entries(&mut self, entries: Vec<SearchResultEntry>) {
        for SearchResultEntry { icon, text, data } in entries {
            if let Some(data) = data {
                self.browser.add_with_data(&text, data);
            } else {
                self.browser.add(&text);
            }

            self.browser.set_icon(self.browser.size(), icon);
        }
    }
}
