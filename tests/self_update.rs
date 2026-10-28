use rudis_lite::self_update::is_newer;

#[test]
fn compares_release_versions_without_downgrading() {
    assert!(is_newer("0.1.0", "0.2.0"));
    assert!(is_newer("0.1.9", "1.0.0"));
    assert!(!is_newer("0.2.0", "v0.2.0"));
    assert!(!is_newer("1.0.0", "0.9.9"));
}
