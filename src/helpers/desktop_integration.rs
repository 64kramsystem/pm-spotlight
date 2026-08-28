use std::path::Path;

use super::{clipboard_management::copy_to_clipboard, file_execution::open_path};

pub trait DesktopIntegration: Send + Sync {
    fn copy_text(&self, text: String) -> Result<(), String>;
    fn open_path(&self, path: &Path) -> Result<(), String>;
}

pub struct NativeDesktopIntegration;

impl DesktopIntegration for NativeDesktopIntegration {
    fn copy_text(&self, text: String) -> Result<(), String> {
        copy_to_clipboard(text)
    }

    fn open_path(&self, path: &Path) -> Result<(), String> {
        open_path(path)
    }
}
