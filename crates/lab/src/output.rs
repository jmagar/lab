//! Output formatting for CLI commands.

pub mod render;
pub mod theme;

pub use render::{print, render_audit_report, render_scaffold_result};
pub use theme::{ColorPolicy, OutputFormat, RenderEnv, human_output_styling_enabled};
