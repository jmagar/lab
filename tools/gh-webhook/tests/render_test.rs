use std::path::PathBuf;

use gh_webhook::github::{Comment, User};
use gh_webhook::render::{fence_for, is_safe_path_component, render_digest, safe_output_path};

fn comment(body: &str) -> Comment {
    Comment {
        id: 1,
        user: User { login: "octocat".into() },
        body: body.into(),
        created_at: "2026-04-20T00:00:00Z".into(),
        updated_at: "2026-04-20T00:00:00Z".into(),
        html_url: "https://github.com/o/r/pull/1#discussion_r1".into(),
        path: None,
        line: None,
    }
}

#[test]
fn safe_path_component_accepts_safe_names() {
    assert!(is_safe_path_component("jmagar"));
    assert!(is_safe_path_component("lab-apis"));
    assert!(is_safe_path_component("a.b_c-1"));
    assert!(is_safe_path_component("A"));
}

#[test]
fn safe_path_component_rejects_empty() {
    assert!(!is_safe_path_component(""));
}

#[test]
fn safe_path_component_rejects_dot_and_dotdot() {
    assert!(!is_safe_path_component("."));
    assert!(!is_safe_path_component(".."));
}

#[test]
fn safe_path_component_rejects_slashes_and_null() {
    assert!(!is_safe_path_component("/foo"));
    assert!(!is_safe_path_component("a/b"));
    assert!(!is_safe_path_component("a\\b"));
    assert!(!is_safe_path_component("a\0b"));
}

#[test]
fn safe_path_component_rejects_overlong() {
    let s: String = "a".repeat(101);
    assert!(!is_safe_path_component(&s));
    let ok: String = "a".repeat(100);
    assert!(is_safe_path_component(&ok));
}

#[test]
fn safe_path_component_rejects_non_ascii() {
    // Cyrillic 'а' homoglyph
    assert!(!is_safe_path_component("\u{0430}bc"));
    // Emoji
    assert!(!is_safe_path_component("file\u{1F600}"));
    // Latin extended
    assert!(!is_safe_path_component("café"));
}

#[test]
fn safe_output_path_blocks_traversal() {
    let root = PathBuf::from("/tmp/x");
    assert!(safe_output_path(&root, "..").is_err());
    assert!(safe_output_path(&root, ".").is_err());
    assert!(safe_output_path(&root, "a/b").is_err());
    assert!(safe_output_path(&root, "").is_err());
    assert!(safe_output_path(&root, "ok.md").is_ok());
}

#[test]
fn fence_for_minimum_three() {
    assert_eq!(fence_for("no backticks").len(), 3);
    assert_eq!(fence_for(""), "```");
}

#[test]
fn fence_for_longer_than_longest_run() {
    // single ` → fence of 3
    assert_eq!(fence_for("a `x` b"), "```");
    // run of 3 → fence of 4
    assert_eq!(fence_for("pre ``` post"), "````");
    // run of 4 → fence of 5
    assert_eq!(fence_for("pre ```` post"), "`````");
    // run of 6 → fence of 7
    assert_eq!(fence_for("``````"), "```````");
}

#[test]
fn fence_for_tracks_max_run() {
    // mixed runs; max is 5
    let s = "a ``` b ````` c ``";
    assert_eq!(fence_for(s), "``````"); // 6 backticks
}

#[test]
fn render_digest_fences_longer_than_body_backticks() {
    let out = render_digest(&[comment("hello\n```malicious```\n")]);
    // must contain a 4+ backtick fence since body has a run of 3
    assert!(out.contains("````"));
    assert!(out.contains("hello"));
    assert!(out.contains("@octocat"));
}

#[test]
fn render_digest_path_uses_code_fence_not_span() {
    let mut c = comment("body");
    c.path = Some("weird`name.rs".to_string());
    c.line = Some(42);
    let out = render_digest(&[c]);
    // path must appear; must NOT be wrapped by single-backtick span.
    // It should appear inside a fence block (fence length >=4 because path has one backtick run of 1 → fence is 3 min).
    // Ensure the path line is not rendered as `weird`name.rs` (single-backtick span).
    assert!(out.contains("weird`name.rs"));
    assert!(!out.contains("`weird`name.rs`"));
    // Should include line number
    assert!(out.contains(":42"));
}

#[test]
fn render_digest_path_with_triple_backticks_uses_longer_fence() {
    let mut c = comment("body");
    c.path = Some("evil```name.rs".to_string());
    c.line = Some(7);
    let out = render_digest(&[c]);
    assert!(out.contains("evil```name.rs"));
    // fence for this path must be 4+ backticks
    assert!(out.contains("````"));
}

#[test]
fn render_digest_non_https_url_not_rendered_as_link() {
    let mut c = comment("body");
    c.html_url = "javascript:alert(1)".into();
    let out = render_digest(&[c]);
    assert!(!out.contains("[view on github](javascript:"));
    assert!(!out.contains("](javascript:"));

    let mut c2 = comment("body");
    c2.html_url = "http://example.com/x".into();
    let out2 = render_digest(&[c2]);
    assert!(!out2.contains("](http://"));
}

#[test]
fn render_digest_https_url_is_linked() {
    let c = comment("body");
    let out = render_digest(&[c]);
    assert!(out.contains("[view on github](https://github.com/o/r/pull/1#discussion_r1)"));
}
