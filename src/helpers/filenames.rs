use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use regex::Regex;

// Map full filenames to unique short names, adding parents where required:
//
//   [a/b/c/d, a/b/d/d, e/f] -> {c/d => a/b/c/d, d/d => a/b/d/d, f => e/f}
//
//
pub fn map_filenames_to_short_names(filenames: Vec<String>) -> HashMap<String, String> {
    let filenames = filenames.into_iter().collect::<HashSet<_>>();

    // Working set format: short name -> parent
    //
    let mut working_set = HashMap::new();

    // Result format: short name -> full path
    //
    let mut result = HashMap::new();

    for filename in &filenames {
        let path = Path::new(filename);

        let basename = Path::new(path.file_name().unwrap()).to_path_buf();
        let parent = path.parent().unwrap();

        let current_parents = working_set.entry(basename).or_insert_with(Vec::new);
        current_parents.push(parent);
    }

    while !working_set.is_empty() {
        let shortnames = working_set.keys().cloned().collect::<Vec<_>>();

        for shortname in shortnames {
            let parents = working_set.remove(&shortname).unwrap();

            if parents.len() == 1 {
                let full_filename = parents[0].join(&shortname).to_str().unwrap().to_string();
                let shortname = shortname.to_str().unwrap().to_string();

                result.insert(shortname, full_filename);
            } else {
                for parent in &parents {
                    let new_parent = parent.parent().unwrap();
                    let super_dir = parent.strip_prefix(new_parent).unwrap();
                    let new_shortname = Path::new(super_dir).join(&shortname);

                    let current_parents = working_set.entry(new_shortname).or_insert_with(Vec::new);
                    current_parents.push(new_parent);
                }
            }
        }
    }

    result
}

pub(crate) fn sort_by_basename_match(
    filenames: &mut [(String, String)],
    pattern: &str,
    regex: &Regex,
) {
    let wildcard_only = pattern.chars().all(|character| character == '*');

    filenames.sort_by_cached_key(|(_, fullname)| {
        let basename = Path::new(fullname)
            .file_name()
            .and_then(|basename| basename.to_str())
            .unwrap_or(fullname)
            .to_lowercase();
        let (match_class, position, length) = if wildcard_only {
            (3, usize::MAX, 0)
        } else {
            match regex.find(&basename) {
                Some(found) if found.start() == 0 && found.end() == basename.len() => {
                    (0, 0, basename.chars().count())
                }
                Some(found) if found.start() == 0 => (1, 0, basename.chars().count()),
                Some(found) => (2, found.start(), basename.chars().count()),
                None => (3, usize::MAX, basename.chars().count()),
            }
        };

        (
            match_class,
            position,
            length,
            basename,
            fullname.to_lowercase(),
            fullname.clone(),
        )
    });
}

#[cfg(test)]
mod tests {
    use regex::RegexBuilder;

    use super::sort_by_basename_match;

    fn filename(value: &str) -> (String, String) {
        (value.to_string(), format!("/search/{value}"))
    }

    fn regex(pattern: &str) -> regex::Regex {
        let pattern = pattern
            .split('*')
            .map(regex::escape)
            .collect::<Vec<_>>()
            .join(".*");
        RegexBuilder::new(&pattern)
            .case_insensitive(true)
            .build()
            .unwrap()
    }

    #[test]
    fn basename_matches_are_ranked_by_closeness_then_deterministically() {
        let mut filenames = vec![
            filename("other-mp3"),
            filename("MP3-tools"),
            filename("album.mp3"),
            filename("mp3"),
            filename("my-mp3"),
        ];

        sort_by_basename_match(&mut filenames, "mp3", &regex("mp3"));

        assert_eq!(
            filenames
                .into_iter()
                .map(|(_, fullname)| fullname)
                .collect::<Vec<_>>(),
            [
                "/search/mp3",
                "/search/MP3-tools",
                "/search/my-mp3",
                "/search/album.mp3",
                "/search/other-mp3",
            ]
        );
    }

    #[test]
    fn wildcards_are_ranked_using_the_actual_search_match() {
        let mut filenames = vec![
            filename("zzz-foobar"),
            filename("foo-bar-extra"),
            filename("foo-x-bar"),
        ];

        sort_by_basename_match(&mut filenames, "foo*bar", &regex("foo*bar"));

        assert_eq!(filenames[0].1, "/search/foo-x-bar");
        assert_eq!(filenames[1].1, "/search/foo-bar-extra");
        assert_eq!(filenames[2].1, "/search/zzz-foobar");
    }

    #[test]
    fn wildcard_only_queries_fall_back_to_alphabetical_order() {
        let mut filenames = vec![filename("Zulu"), filename("alpha"), filename("Beta")];

        sort_by_basename_match(&mut filenames, "***", &regex("***"));

        assert_eq!(
            filenames
                .into_iter()
                .map(|(_, fullname)| fullname)
                .collect::<Vec<_>>(),
            ["/search/alpha", "/search/Beta", "/search/Zulu"]
        );
    }

    #[test]
    fn duplicate_basenames_use_the_full_path_as_a_deterministic_tiebreaker() {
        let mut filenames = vec![
            ("z/notes.md".to_string(), "/search/z/notes.md".to_string()),
            ("a/notes.md".to_string(), "/search/A/notes.md".to_string()),
        ];

        sort_by_basename_match(&mut filenames, "notes", &regex("notes"));

        assert_eq!(filenames[0].1, "/search/A/notes.md");
        assert_eq!(filenames[1].1, "/search/z/notes.md");
    }
}
