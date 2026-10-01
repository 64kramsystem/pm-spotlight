use fltk::{
    app::{self, is_event_shift, set_focus, App, Receiver, Sender},
    browser::HoldBrowser,
    dialog,
    enums::{CallbackTrigger, Color, Event, Font, FrameType, Key},
    group::Flex,
    image::PngImage,
    input::Input,
    prelude::*,
    window::Window,
};
use std::sync::Arc;

use crate::{
    application::controller::AppController,
    search::{
        search_manager::SearchManager, search_result_entry::SearchResultEntry,
        searcher::ExecutionAction,
    },
};

use super::{
    message_event::MessageEvent::{self, *},
    selection::selected_entry_index,
};

const WINDOW_TITLE: &str = "Poor Man's Spotlight!";

const WINDOW_WIDTH: i32 = 400;
const WINDOW_HEIGHT: i32 = 500;

const WINDOW_ICON: &[u8] = include_bytes!("../../resources/window_icon/telescope.png");

const BROWSER_TEXT_SIZE: i32 = 15; // default: 14

pub struct PMSpotlightApp {
    controller: AppController,
    app: App,
    sender: Sender<MessageEvent>,
    receiver: Receiver<MessageEvent>,
    browser: HoldBrowser,
    // Row n maps to entries[n - 1], and browser rows borrow their icons from entries.
    // Clear the browser before entries so each icon outlives its row.
    entries: Vec<SearchResultEntry>,
    input: Input,
}

impl PMSpotlightApp {
    pub fn build(search_manager: SearchManager) -> Self {
        let app = App::default().with_scheme(app::Scheme::Gtk);
        app::background(242, 244, 248);
        app::background2(255, 255, 255);
        app::foreground(38, 46, 59);

        let mut window = Window::default()
            .with_size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .with_label(WINDOW_TITLE);
        let mut layout = Flex::default_fill().column();
        layout.set_margin(12);
        layout.set_pad(10);

        Self::set_window_icon(&mut window);

        let (sender, receiver) = app::channel();

        let mut input = Input::default();
        input.set_frame(FrameType::ThinDownBox);
        input.set_text_font(Font::Helvetica);
        input.set_text_size(16);
        input.set_selection_color(Color::from_rgb(204, 223, 250));
        layout.fixed(&input, 40);

        let mut browser = HoldBrowser::default();
        browser.set_frame(FrameType::FlatBox);
        browser.set_text_size(BROWSER_TEXT_SIZE);
        browser.set_selection_color(Color::from_rgb(160, 195, 240));
        browser.set_scrollbar_size(12);
        input.set_trigger(CallbackTrigger::Changed);

        Self::callback_start_search(&mut input, sender);
        Self::fltk_event_list_execute_entry_and_focus_on_browser(&mut input, sender);
        Self::fltk_event_execute_entry_from_browser(&mut browser, sender);

        layout.end();
        window.resizable(&layout);
        window.size_range(320, 220, 0, 0);
        window.end();
        window.show();

        Self {
            controller: AppController::new(search_manager),
            app,
            sender,
            receiver,
            browser,
            entries: Vec::new(),
            input,
        }
    }

    pub fn set_window_icon(window: &mut Window) {
        let image = PngImage::from_data(WINDOW_ICON).unwrap();
        window.set_icon(Some(image));
    }

    pub fn run(&mut self) {
        while self.app.wait() {
            while let Some(event) = self.receiver.recv() {
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
                    ExecuteEntry(alternate) => {
                        if self.message_event_execute_entry(alternate) {
                            self.app.quit();
                            return;
                        }
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
                sender.send(ExecuteEntry(is_event_shift()));
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
                sender.send(ExecuteEntry(is_event_shift()));
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
        self.entries.clear();
        self.controller.start_search(pattern, Arc::new(self.sender));
    }

    fn message_event_update_list(&mut self, entries: Vec<SearchResultEntry>) {
        for entry in self.controller.filter_current_entries(entries) {
            let icon = entry.icon.clone();

            self.browser.add(&entry.label);
            self.browser.set_icon(self.browser.size(), icon);
            self.entries.push(entry);
        }
    }

    fn message_event_focus_on_browser(&mut self) {
        if self.browser.size() > 0 {
            set_focus(&self.browser);
            self.browser.select(1);
        }
    }

    fn message_event_execute_entry(&mut self, alternate: bool) -> bool {
        let Some(selected_index) = selected_entry_index(self.browser.value(), self.browser.size())
        else {
            return false;
        };

        let Some(entry) = self.entries.get(selected_index).cloned() else {
            return false;
        };

        match self.controller.execute_entry(&entry, alternate) {
            Ok(ExecutionAction::ExitApplication) => return true,
            Ok(ExecutionAction::Ignore) => {}
            Err(error) => {
                dialog::alert_default(&format!(
                    "Poor Man's Spotlight could not execute the selected entry:\n\n{error}"
                ));
                set_focus(&self.input);
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{config::config_manager::Config, helpers::desktop_integration::DesktopIntegration};
    use std::{
        path::Path,
        sync::atomic::{AtomicBool, AtomicUsize, Ordering},
        time::{Duration, Instant},
    };

    struct RecordingDesktop {
        fail: AtomicBool,
        calls: AtomicUsize,
    }

    impl DesktopIntegration for RecordingDesktop {
        fn copy_text(&self, _text: String) -> Result<(), String> {
            Ok(())
        }

        fn open_path(&self, _path: &Path) -> Result<(), String> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            if self.fail.load(Ordering::Relaxed) {
                Err("desktop unavailable".into())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    #[ignore = "requires an X display; run with xvfb-run and --ignored --test-threads=1"]
    fn gui_keeps_failed_search_and_stops_draining_messages_after_success() {
        let desktop = Arc::new(RecordingDesktop {
            fail: AtomicBool::new(true),
            calls: AtomicUsize::new(0),
        });
        let manager = SearchManager::with_dependencies(
            Config {
                search_paths: vec![],
                skip_paths: vec![],
            },
            desktop.clone(),
            std::env::temp_dir(),
        )
        .unwrap();
        let mut application = PMSpotlightApp::build(manager);
        application.input.set_value("foo/bar");
        application.message_event_start_search("foo/bar".into());
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(UpdateList(entries)) = application.receiver.recv() {
                assert!(entries.is_empty());
                break;
            }
            assert!(Instant::now() < deadline, "timed out waiting for search");
            app::wait_for(0.01).unwrap();
        }
        application.message_event_update_list(vec![
            SearchResultEntry::new(None, "stale".into(), None, 0, true),
            SearchResultEntry::new(None, "bar".into(), Some("/foo/bar".into()), 1, true),
        ]);
        assert_eq!(application.browser.size(), 1);

        app::add_timeout3(0.01, |_| {
            app::modal().expect("expected error dialog").hide();
        });
        assert!(!application.message_event_execute_entry(false));
        assert_eq!(application.input.value(), "foo/bar");
        assert_eq!(application.browser.size(), 1);
        assert_eq!(application.entries[0].value.as_deref(), Some("/foo/bar"));

        desktop.fail.store(false, Ordering::Relaxed);
        application.sender.send(FocusOnBrowser);
        application.sender.send(ExecuteEntry(false));
        application.sender.send(ExecuteEntry(false));
        application.run();
        assert_eq!(desktop.calls.load(Ordering::Relaxed), 2);
        assert!(matches!(
            application.receiver.recv(),
            Some(ExecuteEntry(false))
        ));
        assert!(!application.app.wait());
    }
}
