use super::is_warp_bundle;

#[test]
fn is_warp_bundle_recognises_warp_channels() {
    assert!(is_warp_bundle("dev.zhangyu1818.warply"));
    assert!(is_warp_bundle("dev.zhangyu1818.warply-dev"));
    assert!(is_warp_bundle("dev.zhangyu1818.warply-preview"));
    assert!(is_warp_bundle("dev.zhangyu1818.warply-local"));
}

#[test]
fn is_warp_bundle_rejects_other_apps() {
    assert!(!is_warp_bundle("com.microsoft.VSCode"));
    assert!(!is_warp_bundle("com.apple.TextEdit"));
    assert!(!is_warp_bundle("dev.zed.Zed"));
    assert!(!is_warp_bundle("invalid"));
    assert!(!is_warp_bundle(""));
}
