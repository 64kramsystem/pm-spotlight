// The main clipboard managers had issues:
//
// - Copypasta 0.71 was unstable; sometimes it didn't copy to clipboard, sometimes it has strange side
//   effects, like not pasting on the first paste invocation, or the paste being displayed in the
//   destination program only after other chars were typed.
// - Clipboard 0.5.0 did not copy emojis on Linux (!)

#[cfg(not(target_os = "linux"))]
use clipboard::{ClipboardContext, ClipboardProvider};

#[cfg(target_os = "linux")]
use std::{
    io::Write,
    process::{Command, Stdio},
};

pub fn copy_to_clipboard(text: String) {
    #[cfg(not(target_os = "linux"))]
    {
        let mut ctx: ClipboardContext =
            ClipboardProvider::new().expect("Failed to initialize clipboard");
        ctx.set_contents(text)
            .expect("Failed to set clipboard contents");
    }
    #[cfg(target_os = "linux")]
    {
        let mut child = Command::new("xsel")
            .arg("-ib")
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let mut child_stdin = child.stdin.take().unwrap();

        write!(child_stdin, "{}", text).unwrap();

        drop(child_stdin);
        child.wait().unwrap();
    }
}
