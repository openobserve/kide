use std::env;
use std::collections::HashMap;
use std::process::Command;
use crate::state::KideConfig;
use crate::k8s::system_diagnostics;

/// Get environment variables from the user's shell (equivalent to shell-env npm package)
/// This solves the issue where GUI apps don't inherit shell environment from dotfiles
pub fn get_shell_environment(_config: &KideConfig) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    // Determine user's default shell
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    
    // Command to source shell profile and print environment
    let mut cmd = Command::new(&shell);
    
    // Use different approaches based on shell type
    if shell.contains("zsh") || shell.contains("bash") {
        cmd.args(["-l", "-i", "-c", "env"]);
    } else {
        // Fallback for other shells
        cmd.args(["-c", "env"]);
    }
    
    // Execute shell command to get environment with timeout
    let output = cmd.output()?;
    
    if !output.status.success() {
        return Err(format!("Shell command failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    // Parse environment variables from output
    let env_output = String::from_utf8(output.stdout)?;
    let mut shell_env = HashMap::new();
    
    for line in env_output.lines() {
        if let Some(eq_pos) = line.find('=') {
            let key = &line[..eq_pos];
            let value = &line[eq_pos + 1..];
            
            // Only include if key looks valid (alphanumeric + underscore)
            if key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                shell_env.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    Ok(shell_env)
}

/// Setup enhanced environment by inheriting shell environment (like Lens does)
/// This solves the common issue where packaged desktop apps don't inherit
/// the full shell environment that includes custom PATH modifications.
pub fn setup_environment(config: &KideConfig) {
    println!("🔧 Setting up shell environment for authentication tools...");
    
    // First, try to get shell environment (like shell-env npm package)
    match get_shell_environment(config) {
        Ok(shell_env) => {
            apply_shell_environment(&shell_env);
            println!("✅ Successfully inherited shell environment");
        }
        Err(e) => {
            println!("⚠️  Could not get shell environment ({e}), falling back to manual PATH enhancement");
            setup_fallback_environment();
        }
    }
    
    // Run system diagnostics to check for file descriptor limits
    system_diagnostics();
}

/// Apply important environment variables from shell
fn apply_shell_environment(shell_env: &HashMap<String, String>) {
    for (key, value) in shell_env.iter() {
        // Only set important environment variables
        match key.as_str() {
            "PATH" | "HOME" | "USER" | "AWS_PROFILE" | "AWS_DEFAULT_REGION" | 
            "GOOGLE_APPLICATION_CREDENTIALS" | "KUBECONFIG" => {
                env::set_var(key, value);
                if key == "PATH" {
                    println!("🔧 Inherited shell PATH: {value}");
                }
            }
            _ if key.starts_with("AWS_") || key.starts_with("GOOGLE_") || 
                 key.starts_with("AZURE_") || key.starts_with("KUBE") => {
                env::set_var(key, value);
            }
            _ => {} // Skip other variables
        }
    }
}

/// Fallback environment setup when shell detection fails
fn setup_fallback_environment() {
    let current_path = env::var("PATH").unwrap_or_default();
    
    let mut additional_paths = vec![
        "/usr/local/bin".to_string(),
        "/opt/homebrew/bin".to_string(),
        "/usr/bin".to_string(),
        "/bin".to_string(),
        "/usr/local/aws-cli/v2/current/bin".to_string(),
        "/snap/bin".to_string(),
        "/usr/local/google-cloud-sdk/bin".to_string(),
    ];
    
    if let Ok(home) = env::var("HOME") {
        additional_paths.push(format!("{home}/.local/bin"));
    }
    
    let mut all_paths: Vec<String> = additional_paths.into_iter()
        .filter(|path| !current_path.contains(path))
        .collect();
    
    if !current_path.is_empty() {
        all_paths.push(current_path);
    }
    
    let enhanced_path = all_paths.join(":");
    env::set_var("PATH", &enhanced_path);
    
    println!("🔧 Fallback PATH: {enhanced_path}");
}