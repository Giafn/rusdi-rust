use rudis_lite::download::{parse_checksums, verify_sha256};

#[test]
fn parses_and_verifies_checksums() {
    let hash = "a".repeat(64);
    let sums = parse_checksums(&format!("{hash}  app\n")).unwrap();
    assert_eq!(sums["app"], hash);
    assert!(parse_checksums("bad app\n").is_err());
    assert!(verify_sha256(
        b"hello",
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    )
    .is_ok());
    assert!(verify_sha256(b"hello", &"0".repeat(64)).is_err());
}
