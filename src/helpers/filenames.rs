use std::{collections::HashMap, path::Path};

// Map full filenames to unique short names, adding parents where required:
//
//   [a/b/c/d, a/b/d/d, e/f] -> {c/d => a/b/c/d, d/d => a/b/d/d, f => e/f}
//
//
pub fn map_filenames_to_short_names(filenames: Vec<String>) -> HashMap<String, String> {
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
