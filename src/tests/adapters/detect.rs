use super::*;
use crate::core::profile::KNOWN_PROGRAMS;

#[test]
fn every_cli_atrium_knows_reaches_an_adapter_of_its_own() {
    // A missing arm here fails silently: the CLI still runs, as an `Unknown`
    // that can say nothing but whether it is alive.
    for program in KNOWN_PROGRAMS {
        assert_eq!(detect(program).id(), program, "{program} falls through to a different adapter");
    }
}

#[test]
fn the_directories_around_a_name_are_ignored() {
    assert_eq!(detect("/usr/local/bin/claude").id(), "claude");
    assert_eq!(detect("/home/x/.local/bin/copilot").id(), "copilot");
}

#[test]
fn anything_else_is_still_held() {
    assert_eq!(detect("bash").id(), "unknown");
    assert_eq!(detect("").id(), "unknown");
}

#[test]
fn every_cli_atrium_knows_calls_back_and_nothing_else_claims_to() {
    for program in KNOWN_PROGRAMS {
        assert_eq!(detect(program).status_source(), StatusSource::Hooks, "{program}");
    }
    assert_eq!(detect("bash").status_source(), StatusSource::Heuristic);
}

#[test]
fn a_generated_file_is_written_whole_or_not_at_all() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("deep").join("plugin.js");

    write_atomically(&path, "first").expect("write");
    write_atomically(&path, "second").expect("rewrite");

    assert_eq!(std::fs::read_to_string(&path).expect("read"), "second");
    let left: Vec<_> = std::fs::read_dir(path.parent().expect("dir")).expect("list").flatten().map(|entry| entry.file_name()).collect();
    assert_eq!(left.len(), 1, "no temporary file left beside it: {left:?}");
}
