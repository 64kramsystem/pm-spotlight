use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: String) {
    // Copypasta was unstable; sometimes it didn't copy to clipboard, sometimes it has strange side
    // effects, like not pasting on the first paste invocation, or the paste being displayed in the
    // destination program only after other chars were typed.
    //
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
