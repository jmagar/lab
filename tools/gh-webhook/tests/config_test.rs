use gh_webhook::config::Config;

#[test]
fn debug_redacts_secrets() {
    let c = Config {
        webhook_secret: "supersecret".into(),
        github_token: "ghp_abc123".into(),
        bind: "127.0.0.1:7891".parse().unwrap(),
        data_dir: "/tmp/x".into(),
        debounce_secs: 30,
    };
    let s = format!("{c:?}");
    assert!(!s.contains("supersecret"), "secret leaked: {s}");
    assert!(!s.contains("ghp_abc123"), "token leaked: {s}");
    assert!(s.contains("[redacted]"));
}

#[test]
fn from_env_reports_missing_required() {
    // SAFETY: test runs serially with cleaned env; edition 2024 requires unsafe for env mutation.
    unsafe {
        std::env::remove_var("GH_WEBHOOK_SECRET");
        std::env::remove_var("GH_WEBHOOK_GITHUB_TOKEN");
    }
    let err = Config::from_env().unwrap_err();
    assert!(err.to_string().contains("GH_WEBHOOK_SECRET"));
}

#[test]
fn from_env_rejects_empty_secret() {
    unsafe {
        std::env::set_var("GH_WEBHOOK_SECRET", "");
        std::env::set_var("GH_WEBHOOK_GITHUB_TOKEN", "x");
    }
    let err = Config::from_env().unwrap_err();
    assert!(err.to_string().contains("empty"));
}
