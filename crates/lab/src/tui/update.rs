//! Binary update tab — implementation in lab-iuk.5.

/// State machine for the self-update tab.
#[derive(Debug, Clone)]
pub enum UpdateState {
    Idle,
    Checking,
    Available { current: String, latest: String },
    Downloading { progress: f32 },
    Verifying,
    Done,
    Error { message: String },
}

impl Default for UpdateState {
    fn default() -> Self {
        Self::Idle
    }
}
