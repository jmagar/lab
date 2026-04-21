use gh_webhook::events::{parse_event, Event};

fn load(name: &str) -> Vec<u8> {
    std::fs::read(format!("tests/fixtures/{name}")).unwrap()
}

#[test]
fn parses_pr_review_comment() {
    let body = load("pr_review_comment.json");
    let ev = parse_event("pull_request_review_comment", &body).unwrap();
    match ev {
        Event::PrComment { owner, repo, pr, branch } => {
            assert_eq!(owner, "jmagar");
            assert_eq!(repo, "lab");
            assert_eq!(pr, 42);
            assert_eq!(branch, "feat/foo");
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn parses_pr_review() {
    let body = load("pr_review_comment.json");
    let ev = parse_event("pull_request_review", &body).unwrap();
    assert!(matches!(ev, Event::PrComment { .. }));
}

#[test]
fn parses_pr_opened() {
    let body = load("pull_request_opened.json");
    let ev = parse_event("pull_request", &body).unwrap();
    assert!(matches!(ev, Event::PrLifecycle { .. }));
}

#[test]
fn parses_workflow_run_failed() {
    let body = load("workflow_run_failed.json");
    let ev = parse_event("workflow_run", &body).unwrap();
    match ev {
        Event::CiFailed { owner, repo, branch, url, .. } => {
            assert_eq!(owner, "jmagar");
            assert_eq!(repo, "lab");
            assert_eq!(branch, "feat/baz");
            assert!(url.contains("runs/9"));
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn ignores_plain_issue_comment() {
    let body = load("issue_comment_plain.json");
    let ev = parse_event("issue_comment", &body).unwrap();
    assert!(matches!(ev, Event::Ignored));
}

#[test]
fn ignores_unknown_event() {
    let body = load("pr_review_comment.json");
    let ev = parse_event("star", &body).unwrap();
    assert!(matches!(ev, Event::Ignored));
}
