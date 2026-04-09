//! HTTP error handling.
//!
//! The canonical error type is `ToolError` from `crate::services::error`.
//! It implements `IntoResponse` with correct status codes and structured
//! JSON envelopes (including `valid`, `hint`, `param` fields).
//!
//! `ApiError` was a duplicate type that serialized a bare `{kind, message}`
//! envelope, dropping structured fields. It has been removed — use
//! `ToolError` directly in all HTTP handlers.

pub use crate::services::error::ToolError;
