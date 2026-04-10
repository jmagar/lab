//! Shared helpers for thin CLI dispatch shims.

use std::future::Future;
use std::process::ExitCode;

use anyhow::Result;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::output::{OutputFormat, print};

/// Run an action-style CLI command and emit the canonical dispatch log shape.
pub async fn run_action_command<F, Fut>(
    service: &'static str,
    action: String,
    params: Value,
    format: OutputFormat,
    dispatch: F,
) -> Result<ExitCode>
where
    F: FnOnce(String, Value) -> Fut,
    Fut: Future<Output = Result<Value, ToolError>>,
{
    let start = std::time::Instant::now();
    let result = dispatch(action.clone(), params).await;
    let elapsed_ms = start.elapsed().as_millis();

    match &result {
        Ok(_) => tracing::info!(surface = "cli", service, action, elapsed_ms, "dispatch ok"),
        Err(e) if e.is_internal() => tracing::error!(
            surface = "cli",
            service,
            action,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
        Err(e) => tracing::warn!(
            surface = "cli",
            service,
            action,
            elapsed_ms,
            kind = e.kind(),
            "dispatch error"
        ),
    }

    let value = result.map_err(|e| {
        anyhow::anyhow!(
            "{}",
            serde_json::to_string(&e).unwrap_or_else(|_| e.to_string())
        )
    })?;
    print(&value, format)?;
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    use super::*;
    use crate::test_support::{SharedBuf, captured_logs};

    #[test]
    fn action_command_logs_cli_success_shape() {
        let _tracing_lock = crate::test_support::TRACING_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("lab=info"))
            .with(
                fmt::layer()
                    .json()
                    .with_writer(buf.clone())
                    .with_ansi(false)
                    .without_time(),
            );

        let _guard = tracing::subscriber::set_default(subscriber);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            run_action_command(
                "unifi",
                "sites.list".to_string(),
                json!({}),
                OutputFormat::Json,
                |_action, _params| async { Ok(json!({"ok": true})) },
            )
            .await
            .unwrap();
        });

        let logs = captured_logs(&buf);
        assert!(logs.contains("\"surface\":\"cli\""));
        assert!(logs.contains("\"service\":\"unifi\""));
        assert!(logs.contains("\"action\":\"sites.list\""));
        assert!(logs.contains("\"elapsed_ms\""));
    }

    #[test]
    fn action_command_logs_cli_failure_kind() {
        let _tracing_lock = crate::test_support::TRACING_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let buf = SharedBuf::default();
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new("lab=warn"))
            .with(
                fmt::layer()
                    .json()
                    .with_writer(buf.clone())
                    .with_ansi(false)
                    .without_time(),
            );

        let _guard = tracing::subscriber::set_default(subscriber);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let err = run_action_command(
                "bytestash",
                "snippets.get".to_string(),
                json!({}),
                OutputFormat::Json,
                |_action, _params| async {
                    Err(ToolError::MissingParam {
                        message: "missing required parameter `id`".into(),
                        param: "id".into(),
                    })
                },
            )
            .await
            .unwrap_err();
            assert!(err.to_string().contains("missing_param"));
        });

        let logs = captured_logs(&buf);
        assert!(logs.contains("\"surface\":\"cli\""));
        assert!(logs.contains("\"service\":\"bytestash\""));
        assert!(logs.contains("\"action\":\"snippets.get\""));
        assert!(logs.contains("\"kind\":\"missing_param\""));
    }
}
