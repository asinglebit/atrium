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
fn only_the_ones_that_call_back_claim_to() {
    assert_eq!(detect("claude").status_source(), StatusSource::Hooks);
    assert_eq!(detect("copilot").status_source(), StatusSource::Hooks);
    assert_eq!(detect("opencode").status_source(), StatusSource::Heuristic);
    assert_eq!(detect("codex").status_source(), StatusSource::Heuristic);
    assert_eq!(detect("bash").status_source(), StatusSource::Heuristic);
}
