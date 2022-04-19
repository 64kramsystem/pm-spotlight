use std::{os::unix::prelude::CommandExt, path::Path, process::Command};

use fltk::app::Sender;
use regex::Regex;
use walkdir::{DirEntry, WalkDir};

use super::{search_result_entry::SearchResultEntry, searcher::Searcher};
use crate::{
    config::config_manager::Config,
    gui::message_event::MessageEvent::{self, UpdateList},
    helpers::filenames::map_filenames_to_short_names,
};

const DISALLOWED_PATH_CHARS: &str = r"[^-\w*_./]";

pub struct FileSearcher {
    search_paths: Vec<(String, usize)>,
    skip_paths: Vec<Regex>,
    stop_search: bool,
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
        }
    }

    fn process_search_path_definition(mut path: &str) -> (String, usize) {
        let mut depth = 255;

        let re = Regex::new(r"(.+)\{(\d)\}$").unwrap();

        if let Some(captures) = re.captures(path) {
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

        let is_hidden_re = Regex::new(r"/\.[^/]+$").unwrap();

        if is_hidden_re.is_match(&fullname) {
            return true;
        }

        self.skip_paths
            .iter()
            .any(|skip_re| skip_re.is_match(&fullname))
    }

    fn include_entry(entry: &DirEntry, pattern: &str) -> Option<String> {
        let path = entry.path();

        if path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase()
            .contains(pattern)
        {
            Some(path.to_str().unwrap().to_string())
        } else {
            None
        }
    }
}

impl Searcher for FileSearcher {
    fn handles(&self, pattern: &str) -> bool {
        let re = Regex::new(DISALLOWED_PATH_CHARS).unwrap();

        if re.is_match(pattern) {
            panic!("Only alphanum and *_-./ are allowed");
        } else {
            true
        }
    }

    fn search(&mut self, pattern: String, sender: Sender<MessageEvent>, search_id: u32) {
        if pattern.chars().collect::<Vec<_>>().len() < 2 {
            return;
        }

        let pattern = &pattern.to_lowercase();

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
            walker
                .into_iter()
                .filter_map(|e| Self::include_entry(&e.unwrap(), pattern))
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
            .map(|(label, fullname)| SearchResultEntry::new(None, label, Some(fullname), search_id))
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
        Command::new("xdg-open").args([filename]).exec();
    }
}
