/**
 * Pod Shell/Terminal API Integration Tests
 * 
 * Tests for the 4 shell/terminal endpoints:
 * - start_pod_shell
 * - send_shell_input
 * - resize_shell
 * - stop_pod_shell
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;
use std::time::Duration;

#[tokio::test]
async fn test_start_pod_shell_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful shell session start - returns session ID
    let session = MockSession::new(&params.pod_name, &params.namespace, params.container_name.as_deref());
    api.set_success_response("start_pod_shell", json!(session.id));
    
    let result = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "cols": 80,
        "rows": 24
    })).await;
    
    assert!(result.is_ok());
    let session_id = result.unwrap();
    assert!(session_id.is_string());
    assert!(!session_id.as_str().unwrap().is_empty());
    
    assert!(api.was_called("start_pod_shell"));
}

#[tokio::test]
async fn test_start_pod_shell_default_container() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock shell start without specific container (uses first container)
    let session = MockSession::new(&params.pod_name, &params.namespace, None);
    api.set_success_response("start_pod_shell", json!(session.id));
    
    let result = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": null,
        "cols": 120,
        "rows": 30
    })).await;
    
    assert!(result.is_ok());
    let session_id = result.unwrap();
    assert!(session_id.is_string());
}

#[tokio::test]
async fn test_start_pod_shell_custom_dimensions() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock shell with custom terminal dimensions
    let session = MockSession::new(&params.pod_name, &params.namespace, params.container_name.as_deref());
    api.set_success_response("start_pod_shell", json!(session.id));
    
    let result = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "cols": 200,
        "rows": 50
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_pod_shell_pod_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock pod not found error
    api.set_error_response("start_pod_shell", &mock_not_found_error());
    
    let result = api.invoke("start_pod_shell", json!({
        "pod_name": "nonexistent-pod",
        "namespace": "default",
        "container_name": "main",
        "cols": 80,
        "rows": 24
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_start_pod_shell_container_not_found() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock container not found error
    api.set_error_response("start_pod_shell", "Container 'nonexistent' not found in pod");
    
    let result = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": "nonexistent",
        "cols": 80,
        "rows": 24
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_start_pod_shell_pod_not_running() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock pod not running error
    api.set_error_response("start_pod_shell", "Cannot exec into pod: pod is not running");
    
    let result = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "cols": 80,
        "rows": 24
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not running"));
}

#[tokio::test]
async fn test_start_pod_shell_insufficient_permissions() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock RBAC permission error
    api.set_error_response("start_pod_shell", "Insufficient permissions to exec into pods");
    
    let result = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": "kube-system",
        "container_name": params.container_name,
        "cols": 80,
        "rows": 24
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Insufficient permissions"));
}

#[tokio::test]
async fn test_start_pod_shell_invalid_dimensions() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock invalid terminal dimensions error
    api.set_error_response("start_pod_shell", "Invalid terminal dimensions: cols and rows must be positive");
    
    let result = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "cols": 0,
        "rows": 0
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid terminal dimensions"));
}

#[tokio::test]
async fn test_send_shell_input_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful input sending
    api.set_success_response("send_shell_input", json!(null));
    
    let session_id = "test-session-12345";
    let command = "ls -la\n";
    
    let result = api.invoke("send_shell_input", json!({
        "session_id": session_id,
        "input": command
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("send_shell_input"));
}

#[tokio::test]
async fn test_send_shell_input_commands() {
    let mut api = MockTauriApi::new();
    
    // Mock successful command execution
    api.set_success_response("send_shell_input", json!(null));
    
    let session_id = "test-session-12345";
    let commands = vec![
        "pwd\n",
        "whoami\n",
        "ps aux\n",
        "cat /etc/hostname\n",
        "echo 'Hello World'\n"
    ];
    
    for command in commands {
        let result = api.invoke("send_shell_input", json!({
            "session_id": session_id,
            "input": command
        })).await;
        
        assert!(result.is_ok());
    }
    
    assert_eq!(api.get_call_count("send_shell_input"), 5);
}

#[tokio::test]
async fn test_send_shell_input_interactive_commands() {
    let mut api = MockTauriApi::new();
    
    // Mock interactive command input
    api.set_success_response("send_shell_input", json!(null));
    
    let session_id = "test-session-12345";
    
    // Simulate interactive commands
    let interactive_sequence = vec![
        "vi test.txt\n",          // Start editor
        "i",                      // Insert mode
        "Hello World",            // Type content
        "\x1b",                   // Escape key
        ":wq\n",                  // Save and quit
    ];
    
    for input in interactive_sequence {
        let result = api.invoke("send_shell_input", json!({
            "session_id": session_id,
            "input": input
        })).await;
        
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_send_shell_input_invalid_session() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid session ID error
    api.set_error_response("send_shell_input", "Invalid session ID: nonexistent-session");
    
    let result = api.invoke("send_shell_input", json!({
        "session_id": "nonexistent-session",
        "input": "ls\n"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid session ID"));
}

#[tokio::test]
async fn test_send_shell_input_session_closed() {
    let mut api = MockTauriApi::new();
    
    // Mock session closed error
    api.set_error_response("send_shell_input", "Shell session is closed");
    
    let result = api.invoke("send_shell_input", json!({
        "session_id": "closed-session-123",
        "input": "echo test\n"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("session is closed"));
}

#[tokio::test]
async fn test_send_shell_input_empty_input() {
    let mut api = MockTauriApi::new();
    
    // Empty input should be allowed (for things like Enter key)
    api.set_success_response("send_shell_input", json!(null));
    
    let result = api.invoke("send_shell_input", json!({
        "session_id": "test-session-12345",
        "input": ""
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_resize_shell_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful shell resize
    api.set_success_response("resize_shell", json!(null));
    
    let session_id = "test-session-12345";
    let result = api.invoke("resize_shell", json!({
        "session_id": session_id,
        "cols": 120,
        "rows": 40
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("resize_shell"));
}

#[tokio::test]
async fn test_resize_shell_different_sizes() {
    let mut api = MockTauriApi::new();
    
    // Test various terminal sizes
    let test_sizes = vec![
        (80, 24),   // Standard
        (120, 30),  // Wide
        (100, 50),  // Tall
        (200, 60),  // Large
        (40, 12),   // Small
    ];
    
    api.set_success_response("resize_shell", json!(null));
    
    for (cols, rows) in test_sizes {
        let result = api.invoke("resize_shell", json!({
            "session_id": "test-session-12345",
            "cols": cols,
            "rows": rows
        })).await;
        
        assert!(result.is_ok(), "Failed to resize to {}x{}", cols, rows);
    }
}

#[tokio::test]
async fn test_resize_shell_invalid_session() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid session ID error
    api.set_error_response("resize_shell", "Invalid session ID: invalid-session");
    
    let result = api.invoke("resize_shell", json!({
        "session_id": "invalid-session",
        "cols": 80,
        "rows": 24
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid session ID"));
}

#[tokio::test]
async fn test_resize_shell_invalid_dimensions() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid dimensions error
    api.set_error_response("resize_shell", "Invalid dimensions: cols and rows must be positive");
    
    let result = api.invoke("resize_shell", json!({
        "session_id": "test-session-12345",
        "cols": 0,
        "rows": 24
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid dimensions"));
}

#[tokio::test]
async fn test_stop_pod_shell_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful shell stop
    api.set_success_response("stop_pod_shell", json!(null));
    
    let session_id = "test-session-12345";
    let result = api.invoke("stop_pod_shell", json!({
        "session_id": session_id
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("stop_pod_shell"));
}

#[tokio::test]
async fn test_stop_pod_shell_invalid_session() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid session ID error
    api.set_error_response("stop_pod_shell", "Invalid session ID: nonexistent");
    
    let result = api.invoke("stop_pod_shell", json!({
        "session_id": "nonexistent"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid session ID"));
}

#[tokio::test]
async fn test_stop_pod_shell_already_stopped() {
    let mut api = MockTauriApi::new();
    
    // Mock already stopped session (should succeed silently)
    api.set_success_response("stop_pod_shell", json!(null));
    
    let result = api.invoke("stop_pod_shell", json!({
        "session_id": "already-stopped-session"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stop_pod_shell_empty_session_id() {
    let mut api = MockTauriApi::new();
    
    // Mock error for empty session ID
    api.set_error_response("stop_pod_shell", "Session ID cannot be empty");
    
    let result = api.invoke("stop_pod_shell", json!({
        "session_id": ""
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

// Integration test for full shell session lifecycle
#[tokio::test]
async fn test_shell_session_lifecycle() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Setup responses for full shell lifecycle
    let session = MockSession::new(&params.pod_name, &params.namespace, params.container_name.as_deref());
    api.set_success_response("start_pod_shell", json!(session.id.clone()));
    api.set_success_response("send_shell_input", json!(null));
    api.set_success_response("resize_shell", json!(null));
    api.set_success_response("stop_pod_shell", json!(null));
    
    // Step 1: Start shell session
    let start_result = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "cols": 80,
        "rows": 24
    })).await;
    assert!(start_result.is_ok());
    let session_id = start_result.unwrap().as_str().unwrap().to_string();
    
    // Step 2: Send some commands
    let input_result = api.invoke("send_shell_input", json!({
        "session_id": session_id,
        "input": "ls -la\n"
    })).await;
    assert!(input_result.is_ok());
    
    // Step 3: Resize terminal
    let resize_result = api.invoke("resize_shell", json!({
        "session_id": session_id,
        "cols": 120,
        "rows": 30
    })).await;
    assert!(resize_result.is_ok());
    
    // Step 4: Send more commands
    let input_result2 = api.invoke("send_shell_input", json!({
        "session_id": session_id,
        "input": "pwd\n"
    })).await;
    assert!(input_result2.is_ok());
    
    // Step 5: Stop shell session
    let stop_result = api.invoke("stop_pod_shell", json!({
        "session_id": session_id
    })).await;
    assert!(stop_result.is_ok());
    
    // Verify all operations were called
    assert!(api.was_called("start_pod_shell"));
    assert!(api.was_called("send_shell_input"));
    assert!(api.was_called("resize_shell"));
    assert!(api.was_called("stop_pod_shell"));
    assert_eq!(api.get_call_count("send_shell_input"), 2);
}

// Test multiple concurrent shell sessions
#[tokio::test]
async fn test_multiple_concurrent_shell_sessions() {
    let mut api = MockTauriApi::new();
    
    // Mock multiple successful session starts
    let session1 = MockSession::new("pod-1", "default", Some("container-1"));
    let session2 = MockSession::new("pod-2", "default", Some("container-2"));
    let session3 = MockSession::new("pod-3", "monitoring", Some("container-3"));
    
    api.set_success_response("start_pod_shell", json!(session1.id.clone()));
    
    // Start first session
    let shell1 = api.invoke("start_pod_shell", json!({
        "pod_name": "pod-1",
        "namespace": "default",
        "container_name": "container-1",
        "cols": 80,
        "rows": 24
    })).await;
    assert!(shell1.is_ok());
    
    // Update mock for second session
    api.set_success_response("start_pod_shell", json!(session2.id.clone()));
    
    // Start second session
    let shell2 = api.invoke("start_pod_shell", json!({
        "pod_name": "pod-2",
        "namespace": "default",
        "container_name": "container-2",
        "cols": 100,
        "rows": 30
    })).await;
    assert!(shell2.is_ok());
    
    // Update mock for third session
    api.set_success_response("start_pod_shell", json!(session3.id.clone()));
    
    // Start third session
    let shell3 = api.invoke("start_pod_shell", json!({
        "pod_name": "pod-3",
        "namespace": "monitoring",
        "container_name": "container-3", 
        "cols": 120,
        "rows": 40
    })).await;
    assert!(shell3.is_ok());
    
    // Verify all sessions have different IDs
    let id1 = shell1.unwrap().as_str().unwrap();
    let id2 = shell2.unwrap().as_str().unwrap();
    let id3 = shell3.unwrap().as_str().unwrap();
    
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
    
    assert_eq!(api.get_call_count("start_pod_shell"), 3);
}

// Test shell command execution patterns
#[tokio::test]
async fn test_shell_command_patterns() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("send_shell_input", json!(null));
    
    let session_id = "test-session-12345";
    
    // Test various command patterns
    let commands = vec![
        // Basic commands
        "ls\n",
        "pwd\n", 
        "whoami\n",
        
        // Commands with options
        "ls -la\n",
        "ps aux\n",
        "df -h\n",
        
        // Piped commands
        "ps aux | grep nginx\n",
        "cat /proc/meminfo | head -10\n",
        
        // Multi-line commands
        "echo 'line 1\nline 2\nline 3'\n",
        
        // Commands with special characters
        "echo \"Hello World!\"\n",
        "find . -name '*.txt'\n",
        
        // Control sequences
        "\x03", // Ctrl+C
        "\x04", // Ctrl+D
        "\x1a", // Ctrl+Z
    ];
    
    for command in commands {
        let result = api.invoke("send_shell_input", json!({
            "session_id": session_id,
            "input": command
        })).await;
        
        assert!(result.is_ok(), "Failed to send command: {}", command.escape_debug());
    }
}

// Test terminal resize scenarios
#[tokio::test]
async fn test_terminal_resize_scenarios() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("resize_shell", json!(null));
    
    let session_id = "test-session-12345";
    
    // Test resize sequence (simulating window resizing)
    let resize_sequence = vec![
        (80, 24),   // Initial size
        (120, 30),  // User resizes wider
        (100, 40),  // Resize taller
        (90, 35),   // Resize smaller
        (150, 45),  // Resize larger
        (80, 24),   // Back to standard
    ];
    
    for (cols, rows) in resize_sequence {
        let result = api.invoke("resize_shell", json!({
            "session_id": session_id,
            "cols": cols,
            "rows": rows
        })).await;
        
        assert!(result.is_ok(), "Failed to resize to {}x{}", cols, rows);
    }
    
    assert_eq!(api.get_call_count("resize_shell"), 6);
}

// Test shell session error recovery
#[tokio::test]
async fn test_shell_session_error_recovery() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // First attempt fails
    api.set_error_response("start_pod_shell", "Connection to pod failed");
    
    let first_attempt = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "cols": 80,
        "rows": 24
    })).await;
    assert!(first_attempt.is_err());
    
    // Second attempt succeeds (connection recovered)
    let session = MockSession::new(&params.pod_name, &params.namespace, params.container_name.as_deref());
    api.set_success_response("start_pod_shell", json!(session.id));
    
    let second_attempt = api.invoke("start_pod_shell", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "cols": 80,
        "rows": 24
    })).await;
    assert!(second_attempt.is_ok());
    
    assert_eq!(api.get_call_count("start_pod_shell"), 2);
}

// Performance test for shell responsiveness
#[tokio::test]
async fn test_shell_responsiveness() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("send_shell_input", json!(null));
    
    let session_id = "test-session-12345";
    let start_time = std::time::Instant::now();
    
    // Send rapid commands (simulating fast typing)
    for i in 0..100 {
        let result = api.invoke("send_shell_input", json!({
            "session_id": session_id,
            "input": format!("echo {}\n", i)
        })).await;
        
        assert!(result.is_ok());
    }
    
    let duration = start_time.elapsed();
    
    // Should handle rapid input efficiently
    assert!(duration.as_millis() < 5000);
    assert_eq!(api.get_call_count("send_shell_input"), 100);
}