use arboard::Clipboard;

#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
#[cfg(target_os = "linux")]
use std::{
    ffi::OsStr,
    io::{Read, Write},
    process::{Command, Stdio},
};

#[cfg(target_os = "linux")]
const CLIPBOARD_SERVER_ARG: &str = "--internal-clipboard-server";

pub fn copy_to_clipboard(text: String) {
    #[cfg(target_os = "linux")]
    {
        let mut server = Command::new(std::env::current_exe().unwrap())
            .arg(CLIPBOARD_SERVER_ARG)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();

        server
            .stdin
            .take()
            .unwrap()
            .write_all(text.as_bytes())
            .unwrap();
    }

    #[cfg(not(target_os = "linux"))]
    Clipboard::new().unwrap().set_text(text).unwrap();
}

#[cfg(target_os = "linux")]
pub fn run_clipboard_server_if_requested() -> bool {
    if std::env::args_os().nth(1).as_deref() != Some(OsStr::new(CLIPBOARD_SERVER_ARG)) {
        return false;
    }

    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text).unwrap();
    let mut clipboard = Clipboard::new().unwrap();
    clipboard.set().wait().text(text).unwrap();

    true
}
