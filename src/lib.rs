pub mod application {
    pub mod controller;
}

pub mod config {
    pub mod config_manager;
}

pub mod gui {
    pub mod message_event;
    pub mod pm_spotlight_app;
    pub mod selection;
}

pub mod helpers {
    pub mod clipboard_management;
    pub mod desktop_integration;
    pub mod file_execution;
    pub mod filenames;
}

pub mod search {
    pub mod emoji_searcher;
    pub mod file_searcher;
    pub mod search_manager;
    pub mod search_result_entry;
    pub mod searcher;
}
