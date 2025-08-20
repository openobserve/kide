pub mod shell_validator;
pub mod input_sanitizer;

pub use shell_validator::ShellValidator;
pub use input_sanitizer::InputSanitizer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_validator_blocks_dangerous_commands() {
        let validator = ShellValidator::new();
        
        // Test dangerous patterns
        let dangerous_inputs = [
            "some harmful command"
        ];
        
        for input in &dangerous_inputs {
            let result = validator.validate_input(input);
            assert!(result.is_err(), "Should reject dangerous input: {}", input);
        }
    }
    
    #[test]
    fn test_shell_validator_allows_safe_commands() {
        let validator = ShellValidator::new();
        
        let safe_inputs = [
            "ls",
            "pwd",
            "whoami",
            "echo hello world",
            "cd Documents",
            "mkdir test",
        ];
        
        for input in &safe_inputs {
            let result = validator.validate_input(input);
            assert!(result.is_ok(), "Should allow safe input: {}", input);
        }
    }
    
    #[test]
    fn test_shell_validator_length_limit() {
        let validator = ShellValidator::new();
        let long_input = "a".repeat(2000); // Over the 1024 limit
        
        let result = validator.validate_input(&long_input);
        assert!(result.is_err(), "Should reject input that's too long");
    }
    
    #[test]
    fn test_input_sanitizer_resource_names() {
        let sanitizer = InputSanitizer::new();
        
        // Valid resource names
        assert!(sanitizer.validate_resource_name("my-pod").is_ok());
        assert!(sanitizer.validate_resource_name("app123").is_ok());
        
        // Invalid resource names
        assert!(sanitizer.validate_resource_name("").is_err());
        assert!(sanitizer.validate_resource_name("Pod_Name").is_err()); // underscores not allowed
        assert!(sanitizer.validate_resource_name("-starts-with-dash").is_err());
    }
    
    #[test]
    fn test_input_sanitizer_namespaces() {
        let sanitizer = InputSanitizer::new();
        
        // Valid namespaces
        assert!(sanitizer.validate_namespace("default").is_ok());
        assert!(sanitizer.validate_namespace("kube-system").is_ok());
        assert!(sanitizer.validate_namespace("my-namespace").is_ok());
        
        // Invalid namespaces
        assert!(sanitizer.validate_namespace("").is_err());
        assert!(sanitizer.validate_namespace("Invalid_Namespace").is_err());
        assert!(sanitizer.validate_namespace(&"a".repeat(100)).is_err()); // too long
    }
    
    #[test]
    fn test_input_sanitizer_yaml_content() {
        let sanitizer = InputSanitizer::new();
        
        let valid_yaml = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
spec:
  containers:
  - name: test
    image: nginx
"#;
        
        assert!(sanitizer.validate_yaml_content(valid_yaml).is_ok());
        
        // Invalid YAML
        let invalid_yaml = "invalid: yaml: content: [unclosed";
        assert!(sanitizer.validate_yaml_content(invalid_yaml).is_err());
        
        // Empty YAML
        assert!(sanitizer.validate_yaml_content("").is_err());
    }
    
    #[test]
    fn test_shell_validator_safe_commands() {
        let validator = ShellValidator::new();
        let safe_command = validator.get_initial_command();
        
        // Should return bash with proper terminal setup
        assert_eq!(safe_command.len(), 3);
        assert_eq!(safe_command[0], "/bin/bash");
        assert_eq!(safe_command[1], "-c");
        assert!(safe_command[2].contains("TERM=xterm-256color"));
    }
}