use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SearchLogsRequest {
    pub device_id: String,
    pub query: String,
}
