use std::path::Path;

#[cfg(target_os = "linux")]
pub fn open_path(path: &Path) -> Result<(), String> {
    use std::fs::File;

    use ashpd::desktop::open_uri::OpenFileRequest;

    let file = File::open(path).map_err(|error| error.to_string())?;

    async_io::block_on(async {
        OpenFileRequest::default()
            .send_file(&file)
            .await
            .map_err(|error| error.to_string())?
            .response()
            .map_err(|error| error.to_string())
    })
}

#[cfg(target_os = "macos")]
pub fn open_path(path: &Path) -> Result<(), String> {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSURL};

    let path = path
        .to_str()
        .ok_or_else(|| "path is not valid UTF-8".to_string())?;
    let url = NSURL::fileURLWithPath(&NSString::from_str(path));

    if NSWorkspace::sharedWorkspace().openURL(&url) {
        Ok(())
    } else {
        Err(format!("no application can open {path}"))
    }
}

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt, path::PathBuf};

    use super::*;

    #[test]
    fn non_utf8_paths_are_reported_before_calling_nsworkspace() {
        let path = PathBuf::from(OsString::from_vec(b"/tmp/invalid-\xff".to_vec()));

        assert_eq!(open_path(&path).unwrap_err(), "path is not valid UTF-8");
    }
}
