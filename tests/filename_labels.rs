use std::collections::HashMap;

use pm_spotlight::helpers::filenames::map_filenames_to_short_names;

#[test]
fn empty_input_produces_no_labels() {
    assert!(map_filenames_to_short_names(Vec::new()).is_empty());
}

#[test]
fn unique_basenames_are_used_without_parent_directories() {
    let labels = map_filenames_to_short_names(vec![
        "/one/report.txt".to_string(),
        "/two/notes.md".to_string(),
    ]);

    assert_eq!(
        labels,
        HashMap::from([
            ("report.txt".to_string(), "/one/report.txt".to_string()),
            ("notes.md".to_string(), "/two/notes.md".to_string()),
        ])
    );
}

#[test]
fn duplicate_basenames_include_just_enough_parent_path_to_be_unique() {
    let labels = map_filenames_to_short_names(vec![
        "/one/alpha/report.txt".to_string(),
        "/one/beta/report.txt".to_string(),
        "/two/notes.txt".to_string(),
    ]);

    assert_eq!(
        labels,
        HashMap::from([
            (
                "alpha/report.txt".to_string(),
                "/one/alpha/report.txt".to_string(),
            ),
            (
                "beta/report.txt".to_string(),
                "/one/beta/report.txt".to_string(),
            ),
            ("notes.txt".to_string(), "/two/notes.txt".to_string()),
        ])
    );
}

#[test]
fn labels_expand_through_multiple_shared_parent_names() {
    let labels = map_filenames_to_short_names(vec![
        "/one/alpha/shared/report.txt".to_string(),
        "/one/beta/shared/report.txt".to_string(),
    ]);

    assert_eq!(
        labels,
        HashMap::from([
            (
                "alpha/shared/report.txt".to_string(),
                "/one/alpha/shared/report.txt".to_string(),
            ),
            (
                "beta/shared/report.txt".to_string(),
                "/one/beta/shared/report.txt".to_string(),
            ),
        ])
    );
}

#[test]
fn duplicate_full_paths_are_collapsed() {
    let labels = map_filenames_to_short_names(vec![
        "/one/report.txt".to_string(),
        "/one/report.txt".to_string(),
    ]);

    assert_eq!(
        labels,
        HashMap::from([("report.txt".to_string(), "/one/report.txt".to_string())])
    );
}
