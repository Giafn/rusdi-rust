use rudis_lite::{platform, release::Release};

#[test]
fn maps_supported_targets() {
    assert_eq!(
        platform::for_target("linux", "x86_64").unwrap().asset_name,
        "rudis-lite-linux-x86_64"
    );
    assert_eq!(
        platform::for_target("linux", "aarch64").unwrap().asset_name,
        "rudis-lite-linux-aarch64"
    );
    assert_eq!(
        platform::for_target("windows", "x86_64")
            .unwrap()
            .asset_name,
        "rudis-lite-windows-x86_64.exe"
    );
    assert!(platform::for_target("darwin", "x86_64").is_err());
}

#[test]
fn parses_release_and_requires_assets() {
    let release = Release::parse(r#"{"tag_name":"v0.2.0","assets":[{"name":"SHA256SUMS","browser_download_url":"https://x/sums"}]}"#).unwrap();
    assert_eq!(release.tag_name, "v0.2.0");
    assert!(release.asset("missing").is_err());
}
