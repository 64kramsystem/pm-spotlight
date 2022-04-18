# Poor Man's Spotlight

PMsS is a minimal desktop search service, designed to run with multiple backends, currently:

- configurable filesystem search
- emoji search

![Example](/resources/readme_images/example.png?raw=true)

## Nature of the project

The project is a tool I've written for myself, but it's also a research about writing GUIs in Rust versus Ruby.

The previous version of this project was written [in Ruby](https://github.com/64kramsystem/pm-spotlight-old), but had [very severe limitations](https://github.com/64kramsystem/pm-spotlight-old#status).

My current plan is to rewrite the Ruby project with updated version of the libraries (which solve the problems), and publish an article about writing GUIs in the two languages.

## Basic information

The file search configuration must be stored in `$HOME/.pm-spotlight`, and it's a TOML file with content like this:

```toml
search_paths = [
  "Desktop",
  "/usr/include_path{1}",
]
skip_paths = [
  "skip_path",
]
```

Paths not starting with `/` are relative to `$HOME`. Numbers in braces at the end of `search_path` entry are the search depth.

If the user types a pattern starting with a colon (`:`), the request will be sent to the emoji search backend.

Tapping enter on an entry will:

- file search: execute the file
- emoji: copy the emoji to the clipboard
