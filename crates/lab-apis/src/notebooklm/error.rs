use crate::core::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum NotebookLmError {
    #[error(transparent)]
    Api(#[from] ApiError),
}
