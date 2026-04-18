#![cfg(feature = "deploy")]

use lab::dispatch::deploy;
use serde_json::json;

#[test]
fn catalog_lists_required_actions() {
    let names: Vec<&str> = deploy::ACTIONS.iter().map(|a| a.name).collect();
    for required in ["help", "schema", "config.list", "plan", "run", "rollback"] {
        assert!(names.contains(&required), "missing action: {required}");
    }
}

#[test]
fn run_and_rollback_are_destructive_and_others_are_not() {
    for action in deploy::ACTIONS {
        let expect_destructive = matches!(action.name, "run" | "rollback");
        assert_eq!(
            action.destructive, expect_destructive,
            "{} destructive flag wrong",
            action.name
        );
    }
}

#[tokio::test]
async fn unknown_action_returns_stable_kind() {
    let err = deploy::dispatch("not.a.real.action", json!({}))
        .await
        .unwrap_err();
    assert_eq!(err.kind(), "unknown_action");
}

#[tokio::test]
async fn help_lists_run_and_rollback() {
    let v = deploy::dispatch("help", json!({})).await.unwrap();
    assert!(v.is_object());
    let actions = v["actions"].as_array().expect("actions array");
    let names: Vec<&str> = actions
        .iter()
        .map(|a| a["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"run"));
    assert!(names.contains(&"rollback"));
}

#[test]
fn run_missing_targets_returns_validation_failed() {
    // When the token is set via env_override, the next gate is param parsing,
    // which must reject an empty targets array as validation_failed.
    use std::collections::HashMap;
    let mut overrides = HashMap::new();
    overrides.insert("LAB_DEPLOY_TOKEN".to_string(), "test-token".into());
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let err = lab::dispatch::helpers::with_env_override(overrides, || {
        rt.block_on(deploy::dispatch("run", json!({ "confirm": true })))
    })
    .unwrap_err();
    assert_eq!(err.kind(), "validation_failed");
}
