//! Clean: the unit is private, so the inline module is the only tier that can reach it, and
//! the tree it walks has to exist on disk. `std::fs` and `std::env::temp_dir` stay in-module
//! for that reason; the rest of `std::env` does not.

use std::path::Path;

fn markdown_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".md"))
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn lists_only_the_markdown_files() {
        let root = std::env::temp_dir().join("isolation-clean-fixture-read");
        fs::create_dir_all(&root).expect("a temp dir is creatable");
        fs::write(root.join("b.md"), "b").expect("a temp file is writable");
        fs::write(root.join("a.md"), "a").expect("a temp file is writable");
        fs::write(root.join("notes.txt"), "x").expect("a temp file is writable");

        assert_eq!(markdown_names(&root), vec!["a.md", "b.md"]);

        fs::remove_dir_all(&root).expect("the temp dir is removable");
    }
}
