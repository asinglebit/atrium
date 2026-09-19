use super::*;

/// A lookup that says yes to exactly these paths, so the rules can be checked
/// without a filesystem and without the `PATH` this test binary happens to run
/// with.
fn present(paths: &[&str]) -> impl Fn(&Path) -> bool {
    let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    move |candidate: &Path| paths.contains(&candidate.to_path_buf())
}

fn path(dirs: &str) -> Option<OsString> {
    Some(OsString::from(dirs))
}

fn programs(found: &[Found]) -> Vec<&str> {
    found.iter().map(|found| found.program).collect()
}

#[test]
fn what_is_installed_is_found_and_what_is_not_is_left_out() {
    let found = scan_with(path("/opt/bin:/usr/bin"), present(&["/usr/bin/claude", "/opt/bin/opencode"]));

    assert_eq!(programs(&found), ["claude", "opencode"], "codex is not on this machine: {found:?}");
    assert_eq!(found[0].path, PathBuf::from("/usr/bin/claude"), "where it was found is worth keeping");
}

#[test]
fn the_order_is_atriums_rather_than_paths() {
    // opencode comes first on PATH, but atrium lists claude first either way.
    let found = scan_with(path("/a:/b"), present(&["/a/opencode", "/b/claude"]));

    assert_eq!(programs(&found), ["claude", "opencode"]);
}

#[test]
fn the_first_directory_wins_the_way_a_shell_resolves_a_name() {
    let found = scan_with(path("/first:/second"), present(&["/first/claude", "/second/claude"]));

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, PathBuf::from("/first/claude"));
}

#[test]
fn nothing_installed_is_an_empty_list_rather_than_a_guess() {
    assert!(scan_with(path("/usr/bin"), present(&["/usr/bin/vim"])).is_empty());
}

#[test]
fn no_path_at_all_finds_nothing() {
    assert!(scan_with(None, |_| true).is_empty(), "a machine with no PATH cannot be scanned");
}

#[test]
fn only_something_runnable_counts() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("tempdir");
    // A directory carrying the name, and a file that cannot be run.
    std::fs::create_dir(dir.path().join("claude")).expect("dir");
    std::fs::write(dir.path().join("codex"), "notes").expect("write");
    let runnable = dir.path().join("opencode");
    std::fs::write(&runnable, "#!/bin/sh\n").expect("write");
    std::fs::set_permissions(&runnable, std::fs::Permissions::from_mode(0o755)).expect("chmod");

    let found = scan_with(Some(dir.path().as_os_str().to_owned()), is_executable);

    assert_eq!(programs(&found), ["opencode"], "{found:?}");
}

#[test]
fn a_symlink_is_followed_because_a_version_manager_leaves_one() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().expect("tempdir");
    let real = dir.path().join("opencode-1.2.3");
    std::fs::write(&real, "#!/bin/sh\n").expect("write");
    std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    std::os::unix::fs::symlink(&real, dir.path().join("opencode")).expect("symlink");

    let found = scan_with(Some(dir.path().as_os_str().to_owned()), is_executable);

    assert_eq!(programs(&found), ["opencode"], "a shim is how mise and asdf install one: {found:?}");
}
