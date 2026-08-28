use std::{
    collections::HashSet,
    fs,
    panic::{catch_unwind, AssertUnwindSafe},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use regex::{Regex, RegexBuilder};
use walkdir::{DirEntry, WalkDir};

use super::{
    search_result_entry::SearchResultEntry,
    searcher::{ExecutionAction, SearchResultSink, Searcher},
};
use crate::{
    config::config_manager::Config,
    helpers::{desktop_integration::DesktopIntegration, filenames::map_filenames_to_short_names},
};

const DISALLOWED_PATH_CHARS: &str = r"[^-\w*_. /&']";
const DISALLOWED_CHARS_MESSAGE: &str = "Only alphanum and `*_-. /&` are allowed";
const MIN_CHARS: usize = 2;

struct FileSearchPlan {
    search_paths: Vec<(PathBuf, usize)>,
    skip_paths: Vec<Regex>,
    // It's noticeably slow to instantiate once for each file skip test.
    re_is_hidden: Regex,
}

pub struct FileSearcher {
    desktop: Arc<dyn DesktopIntegration>,
    plan: Arc<FileSearchPlan>,
    cancellation: Option<Arc<AtomicBool>>,
}

impl FileSearcher {
    pub fn new(config: Config, desktop: Arc<dyn DesktopIntegration>) -> Result<Self, String> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| "Could not determine the home directory".to_string())?;
        Self::with_home(config, desktop, home_dir)
    }

    pub fn with_home(
        config: Config,
        desktop: Arc<dyn DesktopIntegration>,
        home_dir: PathBuf,
    ) -> Result<Self, String> {
        let search_paths = config
            .search_paths
            .into_iter()
            .map(|path| Self::process_search_path_definition(&path, &home_dir))
            .collect::<Vec<_>>();

        let skip_paths = config
            .skip_paths
            .iter()
            .map(|path| Self::process_skip_path_definition(path, &home_dir))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            desktop,
            plan: Arc::new(FileSearchPlan {
                search_paths,
                skip_paths,
                re_is_hidden: Regex::new(r"/\.[^/]+$").unwrap(),
            }),
            cancellation: None,
        })
    }

    pub(super) fn new_search(&self) -> Self {
        Self {
            desktop: Arc::clone(&self.desktop),
            plan: Arc::clone(&self.plan),
            cancellation: None,
        }
    }

    fn process_search_path_definition(mut path: &str, home_dir: &Path) -> (PathBuf, usize) {
        let mut depth = 255;

        let re_path_with_depth = Regex::new(r"(.+)\{(\d)\}$").unwrap();

        if let Some(captures) = re_path_with_depth.captures(path) {
            path = captures.get(1).unwrap().as_str();
            depth = captures.get(2).unwrap().as_str().parse().unwrap();
        }

        if Path::new(path).is_absolute() {
            (PathBuf::from(path), depth)
        } else {
            (home_dir.join(path), depth)
        }
    }

    // Everything is converted to an absolute path, that must match in full (wildcards are allowed).
    // Skip paths that match at any level, simply are prefixed with '/*/'.
    // Regexes are defined as case-insensitive.
    //
    fn process_skip_path_definition(path: &str, home_dir: &Path) -> Result<Regex, String> {
        let full_path = if Path::new(path).is_absolute() {
            PathBuf::from(path)
        } else {
            home_dir.join(path)
        };
        let full_path = full_path.to_str().ok_or_else(|| {
            format!("Skip path {path:?} expands to a path that is not valid UTF-8")
        })?;
        let regex = format!("^{}$", Self::wildcard_regex(full_path));

        RegexBuilder::new(&regex)
            .case_insensitive(true)
            .build()
            .map_err(|error| format!("Could not compile skip path {path:?}: {error}"))
    }

    fn wildcard_regex(pattern: &str) -> String {
        pattern
            .split('*')
            .map(regex::escape)
            .collect::<Vec<_>>()
            .join(".*")
    }

    // Skip entry format: (filename, is_basename).
    //
    fn skip_entry(plan: &FileSearchPlan, entry: &DirEntry) -> bool {
        let fullname = if let Some(fullname) = entry.path().to_str() {
            fullname.to_string()
        } else {
            return true;
        };

        if plan.re_is_hidden.is_match(&fullname) {
            return true;
        }

        plan.skip_paths
            .iter()
            .any(|skip_re| skip_re.is_match(&fullname))
    }

    fn include_entry(entry: &DirEntry, re_pattern: &Regex) -> Option<String> {
        let path = entry.path();
        let filename = path.file_name()?.to_str()?;

        if re_pattern.is_match(filename) {
            Some(path.to_str()?.to_string())
        } else {
            None
        }
    }

    fn find_matches(
        plan: &FileSearchPlan,
        re_pattern: &Regex,
        cancellation: &AtomicBool,
    ) -> Option<Vec<String>> {
        let mut matching_fullnames = HashSet::new();

        // Ignore nonexisting search paths; a legitimate use case is, for example, a shared config
        // across multiple machines.
        //
        for (search_path, depth) in plan.search_paths.iter().filter(|(path, _)| path.is_dir()) {
            if cancellation.load(Ordering::Acquire) {
                return None;
            }

            let walker = WalkDir::new(search_path)
                .min_depth(1)
                .max_depth(*depth)
                .into_iter()
                .filter_entry(|entry| {
                    !cancellation.load(Ordering::Acquire) && !Self::skip_entry(plan, entry)
                });

            // We can't filter out+in in a single pass, because if we filter out a directory, WalkDir
            // will stop recursing.
            //
            for entry in walker {
                if cancellation.load(Ordering::Acquire) {
                    return None;
                }

                match entry {
                    Ok(entry) => {
                        if let Some(fullname) = Self::include_entry(&entry, re_pattern) {
                            matching_fullnames.insert(fullname);
                        }
                    }
                    Err(error) => eprintln!("{error:?}"),
                }
            }
        }

        Some(matching_fullnames.into_iter().collect())
    }

    fn cancel_active_search(&mut self) {
        if let Some(cancellation) = self.cancellation.take() {
            cancellation.store(true, Ordering::Release);
        }
    }
}

impl Drop for FileSearcher {
    fn drop(&mut self) {
        self.cancel_active_search();
    }
}

impl Searcher for FileSearcher {
    fn handles(&self, _pattern: &str) -> bool {
        true
    }

    fn search(&mut self, pattern: String, sink: Arc<dyn SearchResultSink>, search_id: u32) {
        // SearchManager stops a searcher before replacing it. Keep direct reuse safe as well.
        self.cancel_active_search();

        let re_disallowed_chars = Regex::new(DISALLOWED_PATH_CHARS).unwrap();

        if re_disallowed_chars.is_match(&pattern) {
            let processed_result = vec![SearchResultEntry::new(
                None,
                DISALLOWED_CHARS_MESSAGE.into(),
                None,
                search_id,
                false,
            )];

            sink.send(processed_result);
            return;
        }

        if pattern.chars().count() < MIN_CHARS {
            return;
        }

        let pattern = Self::wildcard_regex(&pattern);
        let re_pattern = match RegexBuilder::new(&pattern).case_insensitive(true).build() {
            Ok(re_pattern) => re_pattern,
            Err(error) => {
                sink.send(vec![SearchResultEntry::new(
                    None,
                    format!("Could not compile the search pattern: {error}"),
                    None,
                    search_id,
                    false,
                )]);
                return;
            }
        };
        let plan = Arc::clone(&self.plan);
        let cancellation = Arc::new(AtomicBool::new(false));
        self.cancellation = Some(Arc::clone(&cancellation));
        let worker_sink = Arc::clone(&sink);

        let worker = thread::Builder::new()
            .name(format!("file-search-{search_id}"))
            .spawn(move || {
                let search_result = catch_unwind(AssertUnwindSafe(|| {
                    let matching_fullnames = Self::find_matches(&plan, &re_pattern, &cancellation)?;

                    let filename_labels = map_filenames_to_short_names(matching_fullnames);

                    if cancellation.load(Ordering::Acquire) {
                        return None;
                    }

                    Some(
                        filename_labels
                            .into_iter()
                            .map(|(label, fullname)| {
                                SearchResultEntry::new(None, label, Some(fullname), search_id, true)
                            })
                            .collect(),
                    )
                }));

                match search_result {
                    Ok(Some(processed_result)) if !cancellation.load(Ordering::Acquire) => {
                        worker_sink.send(processed_result);
                    }
                    Ok(_) => {}
                    Err(panic) if !cancellation.load(Ordering::Acquire) => {
                        let reason = panic
                            .downcast_ref::<&str>()
                            .copied()
                            .or_else(|| panic.downcast_ref::<String>().map(String::as_str))
                            .unwrap_or("unknown worker failure");

                        worker_sink.send(vec![SearchResultEntry::new(
                            None,
                            format!("Filesystem search failed unexpectedly: {reason}"),
                            None,
                            search_id,
                            false,
                        )]);
                    }
                    Err(_) => {}
                }
            });

        if let Err(error) = worker {
            self.cancellation = None;
            sink.send(vec![SearchResultEntry::new(
                None,
                format!("Could not start the filesystem search: {error}"),
                None,
                search_id,
                false,
            )]);
        }
    }

    fn execute(&self, filename: String) -> Result<ExecutionAction, String> {
        self.desktop
            .open_path(Path::new(&filename))
            .map_err(|error| format!("Could not open {filename:?}: {error}"))?;
        Ok(ExecutionAction::ExitApplication)
    }

    fn alt_execute(&self, filename: String) -> Result<ExecutionAction, String> {
        let canonical_path = fs::canonicalize(&filename)
            .map_err(|error| format!("Could not resolve {filename:?}: {error}"))?
            .to_str()
            .ok_or_else(|| format!("Path is not valid UTF-8: {filename:?}"))?
            .to_string();

        self.desktop
            .copy_text(canonical_path)
            .map_err(|error| format!("Could not copy the path for {filename:?}: {error}"))?;

        Ok(ExecutionAction::ExitApplication)
    }

    fn stop(&mut self) {
        self.cancel_active_search();
    }
}
