use clipboard::{ClipboardContext, ClipboardProvider};

// Copypasta was unstable; sometimes it didn't copy to clipboard, sometimes it has strange side
// effects, like not pasting on the first paste invocation, or the paste being displayed in the
// destination program only after other chars were typed.
//
pub fn copy_to_clipboard(text: String) {
    let mut ctx: ClipboardContext =
        ClipboardProvider::new().expect("Failed to initialize clipboard");
    ctx.set_contents(text)
        .expect("Failed to set clipboard contents");
}
