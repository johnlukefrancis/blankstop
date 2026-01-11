mod detect;
mod detect_helpers;
mod continuation;
mod heredoc;
mod reflow;

pub use detect::looks_shell_like;
pub use reflow::reflow_shell;

#[cfg(test)]
mod tests;
