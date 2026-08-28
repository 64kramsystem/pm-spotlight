use pm_spotlight::{
    config::config_manager::ConfigManager, gui::pm_spotlight_app::PMSpotlightApp,
    search::search_manager::SearchManager,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    #[cfg(target_os = "linux")]
    if let Some(result) =
        pm_spotlight::helpers::clipboard_management::run_clipboard_server_if_requested()
    {
        return match result {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("Clipboard server failed: {error}");
                ExitCode::FAILURE
            }
        };
    }

    let config = match ConfigManager::load_configuration() {
        Ok(config) => config,
        Err(error) => {
            report_startup_error(&error);
            return ExitCode::FAILURE;
        }
    };
    let search_manager = match SearchManager::new(config) {
        Ok(search_manager) => search_manager,
        Err(error) => {
            report_startup_error(&error);
            return ExitCode::FAILURE;
        }
    };
    PMSpotlightApp::build(search_manager).run();
    ExitCode::SUCCESS
}

fn report_startup_error(error: &str) {
    let message = format!("Poor Man's Spotlight could not start:\n\n{error}");

    eprintln!("{message}");

    if std::env::var_os("DISPLAY").is_some_and(|display| !display.is_empty()) {
        let _app = fltk::app::App::default();
        fltk::dialog::alert_default(&message);
    }
}
