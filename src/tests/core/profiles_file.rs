use super::*;

fn work() -> StoredProfile {
    StoredProfile {
        name: "work".to_owned(),
        program: String::new(),
        config_dir: "~/.claude-work".to_owned(),
        args: vec!["--append-system-prompt-file".to_owned(), "$DOTFILES/p.md".to_owned()],
        env: Vec::new(),
    }
}

fn both() -> StoredProfiles {
    StoredProfiles { default: "work".to_owned(), profiles: vec![work(), StoredProfile::named("personal")] }
}

#[test]
fn a_file_round_trips() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("profiles.json");

    save_to(&path, &both()).expect("save");

    assert_eq!(load_from(&path), both());
}

#[test]
fn what_is_written_stays_raw_so_the_file_is_portable() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("profiles.json");

    save_to(&path, &both()).expect("save");
    let text = std::fs::read_to_string(&path).expect("read");

    assert!(text.contains("~/.claude-work"), "an expanded home directory would hardcode one machine:\n{text}");
    assert!(text.contains("$DOTFILES"), "an expanded variable would do the same:\n{text}");
}

#[test]
fn a_missing_or_malformed_file_falls_back_rather_than_failing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("nothing.json");
    let broken = dir.path().join("broken.json");
    std::fs::write(&broken, "{ not json").expect("write");

    assert_eq!(load_from(&missing), StoredProfiles::default());
    assert_eq!(load_from(&broken), StoredProfiles::default());
}

#[test]
fn saving_creates_the_directory_it_needs() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("atrium").join("profiles.json");

    save_to(&path, &both()).expect("save");

    assert!(path.exists());
}

#[test]
fn the_file_sits_beside_the_others_atrium_writes() {
    let path = path();

    assert_eq!(path.file_name().and_then(|name| name.to_str()), Some("profiles.json"));
    assert_eq!(path.parent().and_then(|dir| dir.file_name()).and_then(|name| name.to_str()), Some("atrium"));
}

#[test]
fn resolving_expands_what_was_stored_raw() {
    let resolved = work().resolve();

    assert_eq!(resolved.name, "work");
    assert_eq!(resolved.program, "claude", "an empty program means the subscription case");
    assert!(resolved.config_dir().is_some_and(|dir| !dir.starts_with('~')), "{:?}", resolved.config_dir());
}

#[test]
fn an_empty_file_resolves_to_the_clis_atrium_knows() {
    let (profiles, default) = StoredProfiles::default().resolve();

    assert_eq!(profiles, crate::core::profile::defaults());
    assert_eq!(default, 0);
}

#[test]
fn the_default_is_held_by_name_and_found_by_position() {
    let (_, default) = both().resolve();
    assert_eq!(default, 0);

    let mut profiles = both();
    profiles.set_default(1);
    assert_eq!(profiles.resolve().1, 1);
}

#[test]
fn a_default_naming_nothing_falls_back_to_the_first() {
    let profiles = StoredProfiles { default: "gone".to_owned(), profiles: vec![work()] };

    assert_eq!(profiles.default_index(), 0);
}

#[test]
fn the_first_profile_added_becomes_the_default() {
    let mut profiles = StoredProfiles::default();

    profiles.add(StoredProfile::named("work")).expect("add");

    assert_eq!(profiles.default, "work");
}

#[test]
fn a_name_already_taken_is_refused() {
    let mut profiles = both();

    assert!(profiles.add(StoredProfile::named("work")).is_err());
    assert_eq!(profiles.profiles.len(), 2, "nothing should have been appended");
}

#[test]
fn a_profile_needs_a_name() {
    let mut profiles = StoredProfiles::default();

    assert!(profiles.add(StoredProfile::named("  ")).is_err());
    assert!(profiles.rename(0, "   ").is_err());
}

#[test]
fn renaming_keeps_its_place_and_carries_the_default_with_it() {
    let mut profiles = both();

    profiles.rename(0, "job").expect("rename");

    assert_eq!(profiles.profiles[0].name, "job", "it should not have moved");
    assert_eq!(profiles.default, "job", "the default is held by name, so it has to follow");
}

#[test]
fn renaming_onto_another_name_is_refused() {
    let mut profiles = both();

    assert!(profiles.rename(0, "personal").is_err());
    assert_eq!(profiles.profiles[0].name, "work");
}

#[test]
fn renaming_to_the_name_it_already_has_is_allowed() {
    let mut profiles = both();

    assert!(profiles.rename(0, "work").is_ok(), "a no-op edit should not be treated as a collision");
}

#[test]
fn removing_the_default_moves_it_to_what_is_left() {
    let mut profiles = both();

    profiles.remove(0);

    assert_eq!(profiles.profiles.len(), 1);
    assert_eq!(profiles.default, "personal");
}

#[test]
fn removing_the_last_one_leaves_the_built_in_list() {
    let mut profiles = both();

    profiles.remove(0);
    profiles.remove(0);

    assert!(profiles.is_empty());
    assert_eq!(profiles.resolve().0, crate::core::profile::defaults(), "the picker must still have something to offer");
}

#[test]
fn removing_something_that_is_not_there_does_nothing() {
    let mut profiles = both();

    profiles.remove(99);

    assert_eq!(profiles.profiles.len(), 2);
}
