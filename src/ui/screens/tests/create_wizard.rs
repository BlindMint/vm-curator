use super::*;
use crate::app::CreateWizardState;

#[test]
fn test_parse_size_with_suffix_memory() {
    // Plain number assumes target unit (MB)
    assert_eq!(parse_size_with_suffix("8192", "MB"), Some(8192));
    assert_eq!(parse_size_with_suffix("2048", "MB"), Some(2048));

    // GB to MB conversion
    assert_eq!(parse_size_with_suffix("8GB", "MB"), Some(8192));
    assert_eq!(parse_size_with_suffix("8gb", "MB"), Some(8192));  // case insensitive
    assert_eq!(parse_size_with_suffix("32GB", "MB"), Some(32768));
    assert_eq!(parse_size_with_suffix("96GB", "MB"), Some(98304));  // exceeds old 64GB limit
    assert_eq!(parse_size_with_suffix("1024GB", "MB"), Some(1048576));  // 1TB

    // MB to MB (no conversion)
    assert_eq!(parse_size_with_suffix("8192MB", "MB"), Some(8192));

    // KB to MB conversion
    assert_eq!(parse_size_with_suffix("8388608KB", "MB"), Some(8192));

    // Whitespace handling
    assert_eq!(parse_size_with_suffix("  8192  ", "MB"), Some(8192));
    assert_eq!(parse_size_with_suffix("8 GB", "MB"), Some(8192));
}

#[test]
fn test_parse_size_with_suffix_disk() {
    // Plain number assumes target unit (GB)
    assert_eq!(parse_size_with_suffix("500", "GB"), Some(500));
    assert_eq!(parse_size_with_suffix("100", "GB"), Some(100));

    // GB to GB (no conversion)
    assert_eq!(parse_size_with_suffix("500GB", "GB"), Some(500));
    assert_eq!(parse_size_with_suffix("500gb", "GB"), Some(500));

    // MB to GB conversion
    assert_eq!(parse_size_with_suffix("512000MB", "GB"), Some(500));
    assert_eq!(parse_size_with_suffix("1024MB", "GB"), Some(1));
}

#[test]
fn test_parse_size_with_suffix_invalid() {
    // Empty string
    assert_eq!(parse_size_with_suffix("", "MB"), None);

    // Non-numeric
    assert_eq!(parse_size_with_suffix("abc", "MB"), None);
    assert_eq!(parse_size_with_suffix("GB", "MB"), None);

    // Negative values
    assert_eq!(parse_size_with_suffix("-100", "MB"), None);
}

#[test]
fn test_auto_launch_defaults_follow_media_source() {
    let mut state = CreateWizardState::default();
    state.sync_auto_launch_default();
    assert!(!state.auto_launch, "blank new-disk flow should not auto-launch by default");

    state.use_existing_disk = true;
    state.sync_auto_launch_default();
    assert!(state.auto_launch, "existing disk should default to auto-launch");

    let mut iso_state = CreateWizardState::default();
    iso_state.iso_path = Some("/tmp/test.iso".into());
    iso_state.sync_auto_launch_default();
    assert!(iso_state.auto_launch, "install media should default to auto-launch");
}

#[test]
fn test_auto_launch_manual_override_persists() {
    let mut state = CreateWizardState::default();
    state.use_existing_disk = true;
    state.sync_auto_launch_default();
    assert!(state.auto_launch);

    state.toggle_auto_launch();
    assert!(!state.auto_launch);
    assert!(state.auto_launch_overridden);

    state.iso_path = Some("/tmp/test.iso".into());
    state.sync_auto_launch_default();
    assert!(
        !state.auto_launch,
        "manual override should survive later source/media changes"
    );
}
