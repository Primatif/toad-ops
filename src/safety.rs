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

/// Checks if a command name implies a stack that doesn't match the project stack.
pub fn is_stack_mismatch(command: &str, project_stack: &str) -> bool {
    let cmd_lower = command.to_lowercase();
    let stack_lower = project_stack.to_lowercase();

    if cmd_lower.starts_with("cargo ") && !stack_lower.contains("rust") {
        return true;
    }
    if (cmd_lower.starts_with("npm ") || cmd_lower.starts_with("pnpm ") || cmd_lower.starts_with("yarn ")) 
        && !(stack_lower.contains("node") || stack_lower.contains("javascript") || stack_lower.contains("typescript")) {
        return true;
    }
    if cmd_lower.starts_with("go ") && !stack_lower.contains("go") {
        return true;
    }
    if (cmd_lower.starts_with("python ") || cmd_lower.starts_with("pip ") || cmd_lower.starts_with("poetry ")) 
        && !stack_lower.contains("python") {
        return true;
    }

    false
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
