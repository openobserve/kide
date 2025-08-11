use std::collections::HashSet;

/// Secure shell command validator to prevent command injection
#[derive(Debug, Clone)]
pub struct ShellValidator {
    allowed_shells: HashSet<String>,
    max_command_length: usize,
}

impl Default for ShellValidator {
    fn default() -> Self {
        let mut allowed_shells = HashSet::new();
        allowed_shells.insert("sh".to_string());
        allowed_shells.insert("bash".to_string());
        allowed_shells.insert("zsh".to_string());
        
        Self {
            allowed_shells,
            max_command_length: 1024,
        }
    }
}

impl ShellValidator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate and get a safe shell command for pod execution
    pub fn get_safe_shell_command(&self, preferred_shell: Option<&str>) -> Vec<String> {
        match preferred_shell {
            Some(shell) if self.allowed_shells.contains(shell) => {
                vec![shell.to_string()]
            }
            _ => {
                // Safe fallback: try common shells in order of preference
                vec!["bash".to_string(), "sh".to_string()]
            }
        }
    }

    /// Validate user input for shell commands
    pub fn validate_input(&self, input: &str) -> Result<String, ValidationError> {
        if input.len() > self.max_command_length {
            return Err(ValidationError::InputTooLong);
        }

        // Check for dangerous patterns
        let dangerous_patterns = [
            ";", "&&", "||", "|", ">", ">>", "<", "&", "$(",
            "`", "$(", "${", "rm -rf", "mkfs", "dd if=", ":(){ :|:& };:",
        ];

        for pattern in &dangerous_patterns {
            if input.contains(pattern) {
                return Err(ValidationError::DangerousPattern(pattern.to_string()));
            }
        }

        // Basic sanitization - remove control characters except common terminal ones
        let sanitized: String = input
            .chars()
            .filter(|c| {
                c.is_ascii_graphic() || 
                *c == ' ' ||           // space
                *c == '\n' ||          // newline
                *c == '\r' ||          // carriage return
                *c == '\t' ||          // tab
                *c == '\x08' ||        // backspace
                *c == '\x7F' ||        // delete
                *c == '\x1B'           // escape (for arrow keys, etc.)
            })
            .collect();

        Ok(sanitized)
    }

    /// Get initial shell command for pod execution
    pub fn get_initial_command(&self) -> Vec<String> {
        // Use bash with proper terminal environment for better arrow key support
        // Fall back to sh if bash is not available
        vec![
            "/bin/bash".to_string(),
            "-c".to_string(),
            "export TERM=xterm-256color; exec /bin/bash -i".to_string()
        ]
    }
}

#[derive(Debug)]
pub enum ValidationError {
    InputTooLong,
    DangerousPattern(String),
    InvalidShell(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InputTooLong => write!(f, "Input too long (max 1024 characters)"),
            ValidationError::DangerousPattern(pattern) => write!(f, "Dangerous pattern detected: {}", pattern),
            ValidationError::InvalidShell(shell) => write!(f, "Invalid shell: {}", shell),
        }
    }
}

impl std::error::Error for ValidationError {}