use arboard::Clipboard;

#[cfg(target_os = "linux")]
use arboard::SetExtLinux;
#[cfg(target_os = "linux")]
use std::{
    ffi::OsStr,
    io::{BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
    time::Instant,
};

#[cfg(target_os = "linux")]
const CLIPBOARD_SERVER_ARG: &str = "--internal-clipboard-server";

pub fn copy_to_clipboard(text: String) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let executable = std::env::current_exe()
            .map_err(|error| format!("could not locate the current executable: {error}"))?;
        let mut server = Command::new(executable)
            .arg(CLIPBOARD_SERVER_ARG)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("could not start the clipboard server: {error}"))?;

        let result = (|| {
            let mut server_stdin = server
                .stdin
                .take()
                .ok_or_else(|| "clipboard server stdin is unavailable".to_string())?;
            server_stdin
                .write_all(text.as_bytes())
                .map_err(|error| format!("could not send text to the clipboard server: {error}"))?;
            drop(server_stdin);

            let server_stdout = server
                .stdout
                .take()
                .ok_or_else(|| "clipboard server stdout is unavailable".to_string())?;
            let mut response = String::new();
            BufReader::new(server_stdout)
                .read_line(&mut response)
                .map_err(|error| format!("could not read the clipboard server status: {error}"))?;

            match response.trim_end() {
                "OK" => Ok(()),
                response if response.starts_with("ERROR ") => Err(response[6..].to_string()),
                _ => Err("clipboard server exited without a status".to_string()),
            }
        })();

        if result.is_err() {
            let _ = server.wait();
        }

        result
    }

    #[cfg(not(target_os = "linux"))]
    {
        Clipboard::new()
            .map_err(|error| format!("could not access the clipboard: {error}"))?
            .set_text(text)
            .map_err(|error| format!("could not set clipboard text: {error}"))
    }
}

#[cfg(target_os = "linux")]
pub fn run_clipboard_server_if_requested() -> Option<Result<(), String>> {
    if std::env::args_os().nth(1).as_deref() != Some(OsStr::new(CLIPBOARD_SERVER_ARG)) {
        return None;
    }

    let mut text = String::new();
    let result = std::io::stdin()
        .read_to_string(&mut text)
        .map_err(|error| error.to_string())
        .and_then(|_| serve_clipboard(text));

    if let Err(error) = &result {
        let _ = writeln!(std::io::stdout(), "ERROR {error}");
    }

    Some(result)
}

#[cfg(target_os = "linux")]
fn serve_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|error| error.to_string())?;
    clipboard
        .set()
        .wait_until(Instant::now())
        .text(text.clone())
        .map_err(|error| error.to_string())?;

    writeln!(std::io::stdout(), "OK").map_err(|error| error.to_string())?;
    std::io::stdout()
        .flush()
        .map_err(|error| error.to_string())?;

    clipboard
        .set()
        .wait()
        .text(text)
        .map_err(|error| error.to_string())
}
