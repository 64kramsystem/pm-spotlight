use std::{cell::RefCell, rc::Rc};

use fltk::{
    app::{self, event_key_down, focus, set_focus, App, Receiver, Sender},
    browser::HoldBrowser,
    enums::{CallbackTrigger, Key},
    group::Pack,
    image::PngImage,
    input::Input,
    prelude::*,
    window::Window,
};

use crate::search::{searcher::Searcher, searchers_provider::SearchersProvider};

use super::message_event::MessageEvent::{self, *};

type Rcc<T> = Rc<RefCell<T>>;

const WINDOW_TITLE: &str = "Poor Man's Spotlight!";

const WINDOW_WIDTH: i32 = 350;
const WINDOW_HEIGHT: i32 = 500;

const BROWSER_TEXT_SIZE: i32 = 15; // default: 14

pub struct PMSpotlightApp {
    searchers_provider: SearchersProvider,
    current_searcher: Option<Box<dyn Searcher>>,
    app: App,
    receiver: Receiver<MessageEvent>,
    browser: Rcc<HoldBrowser>,
}

impl PMSpotlightApp {
    pub fn build(searchers_provider: SearchersProvider) -> Self {
        let app = App::default();
        let mut window = Window::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .with_label(WINDOW_TITLE);
        let pack = Pack::default().size_of(&window);

        let (sender, receiver) = app::channel();

        let input = Rc::new(RefCell::new(Input::default().with_size(0, 25)));
        let browser = Rc::new(RefCell::new(HoldBrowser::default_fill()));

        browser.borrow_mut().set_text_size(BROWSER_TEXT_SIZE);
        input.borrow_mut().set_trigger(CallbackTrigger::Changed);

        Self::callback_update_list(&input, &sender);
        Self::fltk_event_move_from_input_to_list(&browser, &input);
        Self::fltk_event_select_list_entry(&browser, &input, &sender);

        pack.end();
        window.end();
        window.show();

        Self {
            searchers_provider,
            current_searcher: None,
            app,
            receiver,
            browser,
        }
    }

    pub fn run(&mut self) {
        while self.app.wait() {
            if let Some(event) = self.receiver.recv() {
                match event {
                    UpdateList(pattern) => {
                        self.message_event_update_list(pattern);
                    }
                    SelectListEntry(entry) => {
                        self.message_event_execute_entry(entry);
                    }
                }
            }
        }
    }

    /***************************************************************************
     * Callbacks
     ***************************************************************************/

    fn callback_update_list(input: &Rcc<Input>, sender: &Sender<MessageEvent>) {
        let input = input.clone();
        let sender = sender.clone();

        input.borrow_mut().set_callback(move |input| {
            sender.send(UpdateList(input.value()));
        });
    }

    /***************************************************************************
     * FLTK event handlers
     ***************************************************************************/

    fn fltk_event_move_from_input_to_list(browser: &Rcc<HoldBrowser>, input: &Rcc<Input>) {
        let browser = browser.clone();

        input.borrow_mut().handle(move |input, _| {
            if event_key_down(Key::Down) {
                if let Some(focused) = focus() {
                    if focused.is_same(input) {
                        let mut browser = browser.borrow_mut();

                        if browser.size() > 0 {
                            set_focus(&*browser);
                            browser.select(1);

                            return true;
                        }
                    }
                }
            }

            false
        });
    }

    fn fltk_event_select_list_entry(
        browser: &Rcc<HoldBrowser>,
        input: &Rcc<Input>,
        sender: &Sender<MessageEvent>,
    ) {
        let sender = sender.clone();
        let input = input.clone();

        // It seems that Enter-initiated callback is not supported for browsers.
        //
        browser.borrow_mut().handle(move |browser, _| {
            if let Some(focused) = focus() {
                if focused.is_same(browser) {
                    // An alternative solution was to reset when tapping key up from the topmost Browser entry,
                    // but this is not feasible with fltk(-rs), because:
                    //
                    // - the event is fired after the selection is changed
                    // - the selection doesn't go above the first entry
                    //
                    if event_key_down(Key::Enter) {
                        let selected_line = if browser.value() > 0 {
                            browser.value()
                        } else if browser.size() >= 0 {
                            1
                        } else {
                            return true;
                        };

                        if let Some::<String>(text) = unsafe { browser.data(selected_line) } {
                            sender.send(SelectListEntry(text));
                            Self::reset_interface(browser, &input);
                        } else if let Some(text) = browser.text(selected_line) {
                            sender.send(SelectListEntry(text));
                            Self::reset_interface(browser, &input);
                        }

                        return true;
                    }
                }
            }

            false
        });
    }

    /***************************************************************************
     * MessageEvent handlers
     ***************************************************************************/

    fn message_event_update_list(&mut self, pattern: String) {
        self.browser.borrow_mut().clear();

        self.current_searcher = self.searchers_provider.find_provider(&pattern);

        if let Some(searcher) = &mut self.current_searcher {
            let search_result = searcher.search(&pattern);
            self.set_list_entries(search_result);
        }
    }

    fn message_event_execute_entry(&self, entry: String) {
        if let Some(searcher) = &self.current_searcher {
            searcher.execute(entry);
        }
    }

    /***************************************************************************
     * Helpers
     ***************************************************************************/

    fn set_list_entries(&mut self, entries: Vec<(Option<PngImage>, String, Option<String>)>) {
        let mut browser = self.browser.borrow_mut();

        for (icon, entry_text, entry_data) in entries {
            if let Some(entry_data) = entry_data {
                browser.add_with_data(&entry_text, entry_data);
            } else {
                browser.add(&entry_text);
            }

            let browser_size = browser.size();
            browser.set_icon(browser_size, icon);
        }
    }

    fn reset_interface(browser: &mut HoldBrowser, input: &Rcc<Input>) {
        let mut input = input.borrow_mut();

        input.set_value("");
        set_focus(&*input);
        browser.clear();
    }
}