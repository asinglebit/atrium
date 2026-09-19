use super::*;

#[test]
fn a_saved_width_is_read_back() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("layout.json");

    save_to(&path, &LayoutConfig { sidebar_width: 52 });

    assert_eq!(load_from(&path).sidebar_width, 52);
}

#[test]
fn a_missing_file_is_the_default_rather_than_an_error() {
    let dir = tempfile::tempdir().expect("tempdir");

    assert_eq!(load_from(&dir.path().join("nothing.json")), LayoutConfig::default());
}

#[test]
fn a_malformed_file_falls_back_rather_than_failing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("layout.json");
    std::fs::write(&path, "{ not json").expect("write");

    assert_eq!(load_from(&path), LayoutConfig::default());
}

#[test]
fn saving_creates_the_directory_it_needs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("atrium").join("layout.json");

    save_to(&path, &LayoutConfig { sidebar_width: 30 });

    assert!(path.exists(), "the parent directory should have been created");
}

#[test]
fn the_file_sits_beside_the_theme_atrium_writes() {
    let path = path();

    assert_eq!(path.file_name().and_then(|name| name.to_str()), Some("layout.json"));
    assert_eq!(path.parent().and_then(|dir| dir.file_name()).and_then(|name| name.to_str()), Some("atrium"));
}
