use gh_webhook::hmac::verify_signature;

const SECRET: &str = "It's a Secret to Everybody";
const BODY: &[u8] = b"Hello, World!";
// From GitHub docs: HMAC-SHA256("It's a Secret to Everybody", "Hello, World!")
const SIG: &str = "sha256=757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";

#[test]
fn accepts_valid_signature() {
    assert!(verify_signature(SECRET.as_bytes(), BODY, SIG).is_ok());
}

#[test]
fn rejects_tampered_body() {
    assert!(verify_signature(SECRET.as_bytes(), b"tampered", SIG).is_err());
}

#[test]
fn rejects_missing_prefix() {
    let bad = "757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";
    assert!(verify_signature(SECRET.as_bytes(), BODY, bad).is_err());
}

#[test]
fn rejects_bad_hex() {
    assert!(verify_signature(SECRET.as_bytes(), BODY, "sha256=zzzz").is_err());
}

#[test]
fn rejects_short_sig() {
    assert!(verify_signature(SECRET.as_bytes(), BODY, "sha256=deadbeef").is_err());
}
