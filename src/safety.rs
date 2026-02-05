//! Safety guardrails for destructive bulk operations.

/// Checks if a command string contains potentially destructive patterns.
///
/// Returns `true` if the command is considered "dangerous".
pub fn is_destructive(command: &str) -> bool {
    let dangerous_patterns = [
        "rm ",
        "delete",
        "reset --hard",
        "push -f",
        "force",
        "prune",
        "drop ",
        "truncate",
    ];

    let cmd_lower = command.to_lowercase();
    dangerous_patterns
        .iter()
        .any(|pattern| cmd_lower.contains(pattern))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_destructive() {
        assert!(is_destructive("rm -rf ."));
        assert!(is_destructive("git reset --hard HEAD"));
        assert!(is_destructive("GIT PUSH -F ORIGIN"));
        assert!(!is_destructive("git pull"));
        assert!(!is_destructive("ls -la"));
    }
}
