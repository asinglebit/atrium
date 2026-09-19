//! A guard on the test tree itself.
//!
//! Test files are attached to their source file with `#[cfg(test)] #[path]`,
//! which means a source file rewritten without that block takes its whole test
//! file out of the build -- silently, because the suite still passes. This
//! caught exactly that happening to the sidebar.

use std::{fs, path::Path};

fn rust_files(dir: &Path, found: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            found.push(path);
        }
    }
}

#[test]
fn every_test_file_is_attached_to_a_source_file() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    let mut sources = Vec::new();
    rust_files(&root, &mut sources);
    let all_source_text: String = sources.iter().filter(|path| !path.starts_with(root.join("tests"))).filter_map(|path| fs::read_to_string(path).ok()).collect();

    let mut tests = Vec::new();
    rust_files(&root.join("tests"), &mut tests);
    assert!(!tests.is_empty(), "no test files found at all");

    let orphans: Vec<String> = tests
        .iter()
        .filter_map(|path| path.strip_prefix(root.join("tests")).ok())
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .filter(|rel| rel != "tree.rs" && !all_source_text.contains(&format!("tests/{rel}\"")))
        .collect();

    assert!(orphans.is_empty(), "these test files are not attached to any source file, so they never run: {orphans:?}");
}
