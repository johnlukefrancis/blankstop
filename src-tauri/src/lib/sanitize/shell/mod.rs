mod detect;
mod reflow;

pub use detect::{looks_shell_like, ShellFlavor};
pub use reflow::{reflow_shell, ShellReflowResult, ShellReflowSummary};
