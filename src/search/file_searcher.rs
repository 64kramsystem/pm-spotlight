use std::{fs, os::unix::prelude::CommandExt, path::Path, process::Command};

use fltk::app::Sender;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

use super::{search_result_entry::SearchResultEntry, searcher::Searcher};
use crate::{
    config::config_manager::Config,
    gui::message_event::MessageEvent::{self, UpdateList},
    helpers::{clipboard_management::copy_to_clipboard, filenames::map_filenames_to_short_names},
};

const DISALLOWED_PATH_CHARS: &str = r"[^-\w*_. /&']";
const DISALLOWED_CHARS_MESSAGE: &str = "Only alphanum and `*_-. /&` are allowed";
const MIN_CHARS: usize = 2;

pub struct FileSearcher {
    search_paths: Vec<(String, usize)>,
    skip_paths: Vec<Regex>,
    stop_search: bool,
    // It's noticeably slow to instantiate once for each file skip test.
    re_is_hidden: Regex,
}

impl FileSearcher {
    pub fn new(config: Config) -> Self {
        let search_paths = config
            .search_paths
            .into_iter()
            .map(|path| Self::process_search_path_definition(&path))
            .collect::<Vec<_>>();

        let skip_paths = config
            .skip_paths
            .iter()
            .map(|path| Self::process_skip_path_definition(path))
            .collect::<Vec<_>>();

        Self {
            search_paths,
            skip_paths,
            stop_search: false,
            re_is_hidden: Regex::new(r"/\.[^/]+$").unwrap(),
        }
    }

    fn process_search_path_definition(mut path: &str) -> (String, usize) {
        let mut depth = 255;

        let re_path_with_depth = Regex::new(r"(.+)\{(\d)\}$").unwrap();

        if let Some(captures) = re_path_with_depth.captures(path) {
            path = captures.get(1).unwrap().as_str();
            depth = captures.get(2).unwrap().as_str().parse().unwrap();
        }

        if path.starts_with('/') {
            (path.to_string(), depth)
        } else {
            (
                dirs::home_dir()
                    .unwrap()
                    .join(path)
                    .to_str()
                    .unwrap()
                    .to_string(),
                depth,
            )
        }
    }

    // Everything is converted to an absolute path, that must match in full (wildcards are allowed).
    // Skip paths that match at any level, simply are prefixed with '/*/'.
    // Regexes are defined as case-insensitive.
    //
    fn process_skip_path_definition(path: &str) -> Regex {
        let mut path = path.to_string();

        // Handle home prefix
        //
        if !path.starts_with('/') {
            path = dirs::home_dir()
                .unwrap()
                .join(path)
                .to_str()
                .unwrap()
                .to_string();
        }

        // Handle wildcards.
        //
        let mut regex = path.replace('.', r"\.").replace('*', ".*");

        // Handle full-match and case-sensitivenes
        //
        regex = format!("(?i)^{}$", regex);

        Regex::new(&regex).unwrap()
    }

    // Skip entry format: (filename, is_basename).
    //
    fn skip_entry(&self, entry: &DirEntry) -> bool {
        let fullname = if let Some(fullname) = entry.path().to_str() {
            fullname.to_string()
        } else {
            return true;
        };

        if self.re_is_hidden.is_match(&fullname) {
            return true;
        }

        self.skip_paths
            .iter()
            .any(|skip_re| skip_re.is_match(&fullname))
    }

    fn include_entry(entry: &DirEntry, re_pattern: &Regex) -> Option<String> {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_str().unwrap();

        if re_pattern.is_match(filename) {
            Some(path.to_str().unwrap().to_string())
        } else {
            None
        }
    }
}

impl Searcher for FileSearcher {
    fn handles(&self, _pattern: &str) -> bool {
        true
    }

    fn search(&mut self, pattern: String, sender: Sender<MessageEvent>, search_id: u32) {
        let re_disallowed_chars = Regex::new(DISALLOWED_PATH_CHARS).unwrap();

        if re_disallowed_chars.is_match(&pattern) {
            let processed_result = vec![SearchResultEntry::new(
                None,
                DISALLOWED_CHARS_MESSAGE.into(),
                None,
                search_id,
                false,
            )];

            sender.send(UpdateList(processed_result));
            return;
        }

        if pattern.chars().count() < MIN_CHARS {
            return;
        }

        let mut pattern = pattern.replace('.', r"\.").replace('*', ".*");
        pattern = format!("(?i){}", pattern);
        let re_pattern = Regex::new(&pattern).unwrap();

        let search_in_path = |(search_path, depth): &(String, usize)| {
            let walker = WalkDir::new(search_path)
                .min_depth(1)
                .max_depth(*depth)
                .into_iter()
                .filter_entry(|e| {
                    if self.stop_search {
                        return false;
                    };
                    !self.skip_entry(e)
                });

            // We can't filter out+in in a single pass, because if we filter out a directory, WalkDir will
            // stop recursing.
            //
            walker.into_iter().filter_map(|entry| match entry {
                Ok(entry) => Self::include_entry(&entry, &re_pattern),
                Err(error) => {
                    eprintln!("{:?}", error);
                    None
                }
            })
        };

        // Ignore nonexisting search paths; a legitimate use case is, for example, a shared config
        // across multiple machines.
        //
        let matching_fullnames = self
            .search_paths
            .iter()
            .filter(|(path, _)| Path::new(path).is_dir())
            .flat_map(search_in_path)
            .collect::<Vec<String>>();

        let filename_labels = map_filenames_to_short_names(matching_fullnames);

        let processed_result = filename_labels
            .into_iter()
            .map(|(label, fullname)| {
                SearchResultEntry::new(None, label, Some(fullname), search_id, true)
            })
            .collect();

        sender.send(UpdateList(processed_result));
    }

    fn execute(&self, filename: String) {
        // This is Unix-specific, in two ways:
        //
        // - it uses xdg-open
        // - exec() will replace the pm-spotlight image with the executed program (unless it errors)
        //
        // this is currently fine.
        //
        let _ = Command::new("xdg-open").args([filename]).exec();
    }

    fn alt_execute(&self, filename: String) -> bool {
        let canonical_path = fs::canonicalize(filename)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        copy_to_clipboard(canonical_path);

        std::process::exit(0);
    }
}
