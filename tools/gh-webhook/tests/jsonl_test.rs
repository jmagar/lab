use gh_webhook::jsonl::{NotificationLine, append_line};

fn sample_pr_comments() -> NotificationLine {
    NotificationLine::PrComments {
        owner: "o".into(),
        repo: "r".into(),
        pr: 1,
        branch: "b".into(),
        count: 3,
        digest_path: "/x/latest.md".into(),
        display: "[3] NEW 1 comments for o/r b - View at /x/latest.md".into(),
    }
}

fn sample_ci_failed() -> NotificationLine {
    NotificationLine::CiFailed {
        owner: "o".into(),
        repo: "r".into(),
        branch: "b".into(),
        workflow: "CI".into(),
        run_url: "u".into(),
        display: "[FAIL] CI for o/r b - u".into(),
    }
}

fn sample_pr_lifecycle() -> NotificationLine {
    NotificationLine::PrLifecycle {
        owner: "o".into(),
        repo: "r".into(),
        pr: 7,
        branch: "b".into(),
        action: "opened".into(),
        display: "[PR] opened o/r#7".into(),
    }
}

fn sample_flush_error() -> NotificationLine {
    NotificationLine::FlushError {
        owner: "o".into(),
        repo: "r".into(),
        pr: 7,
        error: "boom".into(),
        display: "[ERR] flush o/r#7: boom".into(),
    }
}

#[test]
fn round_trips_pr_comments() {
    let original = sample_pr_comments();
    let json = serde_json::to_string(&original).unwrap();
    let decoded: NotificationLine = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn append_creates_file_and_appends() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("notifications.jsonl");
    append_line(&path, &sample_pr_comments()).unwrap();
    append_line(&path, &sample_ci_failed()).unwrap();
    let s = std::fs::read_to_string(&path).unwrap();
    let lines: Vec<_> = s.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains("pr_comments"));
    assert!(lines[1].contains("ci_failed"));
    // each line parses as JSON
    for line in &lines {
        let _: serde_json::Value = serde_json::from_str(line).unwrap();
    }
}

#[test]
fn display_field_present() {
    for variant in [
        sample_pr_comments(),
        sample_pr_lifecycle(),
        sample_ci_failed(),
        sample_flush_error(),
    ] {
        let v: serde_json::Value = serde_json::to_value(&variant).unwrap();
        let display = v
            .get("display")
            .and_then(|d| d.as_str())
            .expect("display field must be present and be a string");
        assert!(!display.is_empty(), "display must be non-empty");
    }
}
