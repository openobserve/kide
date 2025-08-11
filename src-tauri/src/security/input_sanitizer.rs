use regex::Regex;
use std::collections::HashMap;

/// Input sanitizer for various Kubernetes resource fields
#[derive(Debug)]
pub struct InputSanitizer {
    resource_name_regex: Regex,
    namespace_regex: Regex,
    label_key_regex: Regex,
    label_value_regex: Regex,
}

impl Default for InputSanitizer {
    fn default() -> Self {
        Self {
            // Kubernetes resource name validation (RFC 1123)
            resource_name_regex: Regex::new(r"^[a-z0-9]([a-z0-9\-]*[a-z0-9])?$").unwrap(),
            // Kubernetes namespace validation
            namespace_regex: Regex::new(r"^[a-z0-9]([a-z0-9\-]*[a-z0-9])?$").unwrap(),
            // Kubernetes label key validation
            label_key_regex: Regex::new(r"^([a-zA-Z0-9]([a-zA-Z0-9\-_.]*[a-zA-Z0-9])?/)?[a-zA-Z0-9]([a-zA-Z0-9\-_.]*[a-zA-Z0-9])?$").unwrap(),
            // Kubernetes label value validation
            label_value_regex: Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9\-_.]*[a-zA-Z0-9])?$|^$").unwrap(),
        }
    }
}

impl InputSanitizer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate Kubernetes resource name
    pub fn validate_resource_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyInput("resource name".to_string()));
        }
        
        if name.len() > 253 {
            return Err(ValidationError::TooLong("resource name".to_string(), 253));
        }

        if !self.resource_name_regex.is_match(name) {
            return Err(ValidationError::InvalidFormat("resource name".to_string()));
        }

        Ok(())
    }

    /// Validate Kubernetes namespace
    pub fn validate_namespace(&self, namespace: &str) -> Result<(), ValidationError> {
        if namespace.is_empty() {
            return Err(ValidationError::EmptyInput("namespace".to_string()));
        }
        
        if namespace.len() > 63 {
            return Err(ValidationError::TooLong("namespace".to_string(), 63));
        }

        if !self.namespace_regex.is_match(namespace) {
            return Err(ValidationError::InvalidFormat("namespace".to_string()));
        }

        Ok(())
    }

    /// Validate YAML content size and basic structure
    pub fn validate_yaml_content(&self, yaml: &str) -> Result<(), ValidationError> {
        if yaml.is_empty() {
            return Err(ValidationError::EmptyInput("YAML content".to_string()));
        }

        // Reasonable size limit for YAML content (1MB)
        if yaml.len() > 1_048_576 {
            return Err(ValidationError::TooLong("YAML content".to_string(), 1_048_576));
        }

        // Basic YAML structure validation - ensure it can be parsed
        match serde_yaml_ng::from_str::<serde_yaml_ng::Value>(yaml) {
            Ok(_) => Ok(()),
            Err(_) => Err(ValidationError::InvalidYaml),
        }
    }

    /// Validate and sanitize labels
    pub fn validate_labels(&self, labels: &HashMap<String, String>) -> Result<(), ValidationError> {
        for (key, value) in labels {
            if key.len() > 316 { // 253 + 63 for prefix/name
                return Err(ValidationError::TooLong("label key".to_string(), 316));
            }
            
            if value.len() > 63 {
                return Err(ValidationError::TooLong("label value".to_string(), 63));
            }

            if !self.label_key_regex.is_match(key) {
                return Err(ValidationError::InvalidFormat("label key".to_string()));
            }

            if !self.label_value_regex.is_match(value) {
                return Err(ValidationError::InvalidFormat("label value".to_string()));
            }
        }
        Ok(())
    }

    /// Sanitize string input by removing potentially dangerous characters
    pub fn sanitize_string(&self, input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || "-_.".contains(*c))
            .take(253) // Max length for most Kubernetes names
            .collect()
    }
}

#[derive(Debug)]
pub enum ValidationError {
    EmptyInput(String),
    TooLong(String, usize),
    InvalidFormat(String),
    InvalidYaml,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyInput(field) => write!(f, "Empty input for {}", field),
            ValidationError::TooLong(field, max) => write!(f, "{} too long (max {} characters)", field, max),
            ValidationError::InvalidFormat(field) => write!(f, "Invalid format for {}", field),
            ValidationError::InvalidYaml => write!(f, "Invalid YAML content"),
        }
    }
}

impl std::error::Error for ValidationError {}