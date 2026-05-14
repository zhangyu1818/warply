use super::*;

#[test]
fn harness_config_name_round_trips_for_known_variants() {
    for harness in [
        Harness::Claude,
        Harness::OpenCode,
        Harness::Gemini,
        Harness::Codex,
    ] {
        assert_eq!(
            Harness::from_config_name(harness.config_name()),
            Some(harness),
            "round-trip failed for {harness:?}",
        );
    }
}

#[test]
fn harness_from_config_name_returns_none_for_unrecognized() {
    assert_eq!(Harness::from_config_name(""), None);
    assert_eq!(Harness::from_config_name("not-a-real-harness"), None);
}
