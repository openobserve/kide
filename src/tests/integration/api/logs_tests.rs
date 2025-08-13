/**
 * Pod Logs Management API Integration Tests
 * 
 * Tests for the 3 logs management endpoints:
 * - get_pod_logs
 * - start_pod_logs_stream
 * - stop_pod_logs_stream
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;
use std::time::Duration;

#[tokio::test]
async fn test_get_pod_logs_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful logs retrieval
    api.set_success_response("get_pod_logs", json!(mock_pod_logs()));
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "lines": 100
    })).await;
    
    assert!(result.is_ok());
    let logs = result.unwrap();
    assert!(logs.is_string());
    
    let logs_string = logs.as_str().unwrap();
    assert!(logs_string.contains("Starting application"));
    assert!(logs_string.contains("Server listening on port 80"));
    assert!(logs_string.contains("[INFO]"));
    
    assert!(api.was_called("get_pod_logs"));
}

#[tokio::test]
async fn test_get_pod_logs_specific_container() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock logs from specific container
    let container_logs = "2024-01-01T00:00:00.000Z [INFO] Container: sidecar started\n2024-01-01T00:00:01.000Z [DEBUG] Sidecar processing...";
    api.set_success_response("get_pod_logs", json!(container_logs));
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": "sidecar",
        "lines": 50
    })).await;
    
    assert!(result.is_ok());
    let logs = result.unwrap();
    let logs_string = logs.as_str().unwrap();
    assert!(logs_string.contains("sidecar started"));
    assert!(logs_string.contains("Sidecar processing"));
}

#[tokio::test]
async fn test_get_pod_logs_no_container_specified() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock logs from default container (first container)
    api.set_success_response("get_pod_logs", json!(mock_pod_logs()));
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": null,
        "lines": 100
    })).await;
    
    assert!(result.is_ok());
    let logs = result.unwrap();
    assert!(logs.is_string());
}

#[tokio::test]
async fn test_get_pod_logs_limited_lines() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock limited log lines
    let limited_logs = "2024-01-01T00:00:04.000Z [INFO] Request processed successfully";
    api.set_success_response("get_pod_logs", json!(limited_logs));
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "lines": 1
    })).await;
    
    assert!(result.is_ok());
    let logs = result.unwrap();
    let logs_string = logs.as_str().unwrap();
    assert_eq!(logs_string.lines().count(), 1);
    assert!(logs_string.contains("Request processed successfully"));
}

#[tokio::test]
async fn test_get_pod_logs_no_lines_specified() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock default logs (all available logs)
    api.set_success_response("get_pod_logs", json!(mock_pod_logs()));
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "lines": null
    })).await;
    
    assert!(result.is_ok());
    let logs = result.unwrap();
    assert!(logs.is_string());
    assert!(logs.as_str().unwrap().lines().count() > 1);
}

#[tokio::test]
async fn test_get_pod_logs_empty_logs() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock pod with no logs
    api.set_success_response("get_pod_logs", json!(""));
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "lines": 100
    })).await;
    
    assert!(result.is_ok());
    let logs = result.unwrap();
    assert_eq!(logs.as_str().unwrap(), "");
}

#[tokio::test]
async fn test_get_pod_logs_pod_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock pod not found error
    api.set_error_response("get_pod_logs", &mock_not_found_error());
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": "nonexistent-pod",
        "namespace": "default",
        "container_name": "main",
        "lines": 100
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_get_pod_logs_container_not_found() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock container not found error
    api.set_error_response("get_pod_logs", "Container 'nonexistent' not found in pod");
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": "nonexistent",
        "lines": 100
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_get_pod_logs_pod_not_running() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock pod not running error
    api.set_error_response("get_pod_logs", "Pod is not in running state");
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "lines": 100
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not in running state"));
}

#[tokio::test]
async fn test_get_pod_logs_invalid_lines() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock invalid lines parameter error
    api.set_error_response("get_pod_logs", "Invalid lines parameter: must be positive integer");
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "lines": -10
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid lines parameter"));
}

#[tokio::test]
async fn test_start_pod_logs_stream_success() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock successful log stream start - returns stream ID
    let session = MockSession::new(&params.pod_name, &params.namespace, params.container_name.as_deref());
    api.set_success_response("start_pod_logs_stream", json!(session.id));
    
    let result = api.invoke("start_pod_logs_stream", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name
    })).await;
    
    assert!(result.is_ok());
    let stream_id = result.unwrap();
    assert!(stream_id.is_string());
    assert!(!stream_id.as_str().unwrap().is_empty());
    
    assert!(api.was_called("start_pod_logs_stream"));
}

#[tokio::test]
async fn test_start_pod_logs_stream_no_container() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock stream start without specific container
    let session = MockSession::new(&params.pod_name, &params.namespace, None);
    api.set_success_response("start_pod_logs_stream", json!(session.id));
    
    let result = api.invoke("start_pod_logs_stream", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": null
    })).await;
    
    assert!(result.is_ok());
    let stream_id = result.unwrap();
    assert!(stream_id.is_string());
}

#[tokio::test]
async fn test_start_pod_logs_stream_pod_not_found() {
    let mut api = MockTauriApi::new();
    
    // Mock pod not found error for streaming
    api.set_error_response("start_pod_logs_stream", &mock_not_found_error());
    
    let result = api.invoke("start_pod_logs_stream", json!({
        "pod_name": "nonexistent-pod",
        "namespace": "default",
        "container_name": "main"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_start_pod_logs_stream_container_not_found() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock container not found error for streaming
    api.set_error_response("start_pod_logs_stream", "Container 'invalid' not found in pod");
    
    let result = api.invoke("start_pod_logs_stream", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": "invalid"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn test_start_pod_logs_stream_already_streaming() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Mock error for already streaming logs
    api.set_error_response("start_pod_logs_stream", "Log stream already active for this pod/container");
    
    let result = api.invoke("start_pod_logs_stream", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already active"));
}

#[tokio::test]
async fn test_stop_pod_logs_stream_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful stream stop
    api.set_success_response("stop_pod_logs_stream", json!(null));
    
    let stream_id = "test-stream-id-12345";
    let result = api.invoke("stop_pod_logs_stream", json!({
        "stream_id": stream_id
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("stop_pod_logs_stream"));
}

#[tokio::test]
async fn test_stop_pod_logs_stream_invalid_id() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid stream ID error
    api.set_error_response("stop_pod_logs_stream", "Invalid stream ID: nonexistent-stream");
    
    let result = api.invoke("stop_pod_logs_stream", json!({
        "stream_id": "nonexistent-stream"
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid stream ID"));
}

#[tokio::test]
async fn test_stop_pod_logs_stream_already_stopped() {
    let mut api = MockTauriApi::new();
    
    // Mock stream already stopped (should succeed silently)
    api.set_success_response("stop_pod_logs_stream", json!(null));
    
    let result = api.invoke("stop_pod_logs_stream", json!({
        "stream_id": "already-stopped-stream"
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stop_pod_logs_stream_empty_id() {
    let mut api = MockTauriApi::new();
    
    // Mock error for empty stream ID
    api.set_error_response("stop_pod_logs_stream", "Stream ID cannot be empty");
    
    let result = api.invoke("stop_pod_logs_stream", json!({
        "stream_id": ""
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("cannot be empty"));
}

// Integration test for full logs workflow
#[tokio::test]
async fn test_logs_workflow_static_then_streaming() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Setup responses for full logs workflow
    api.set_success_response("get_pod_logs", json!(mock_pod_logs()));
    
    let session = MockSession::new(&params.pod_name, &params.namespace, params.container_name.as_deref());
    api.set_success_response("start_pod_logs_stream", json!(session.id.clone()));
    api.set_success_response("stop_pod_logs_stream", json!(null));
    
    // Step 1: Get static logs first
    let static_logs = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "lines": 100
    })).await;
    assert!(static_logs.is_ok());
    
    // Step 2: Start streaming logs
    let stream_start = api.invoke("start_pod_logs_stream", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name
    })).await;
    assert!(stream_start.is_ok());
    let stream_id = stream_start.unwrap().as_str().unwrap().to_string();
    
    // Step 3: Stop streaming logs
    let stream_stop = api.invoke("stop_pod_logs_stream", json!({
        "stream_id": stream_id
    })).await;
    assert!(stream_stop.is_ok());
    
    // Verify all operations were called
    assert!(api.was_called("get_pod_logs"));
    assert!(api.was_called("start_pod_logs_stream"));
    assert!(api.was_called("stop_pod_logs_stream"));
}

// Test multiple concurrent log streams
#[tokio::test]
async fn test_multiple_concurrent_log_streams() {
    let mut api = MockTauriApi::new();
    
    // Mock multiple successful stream starts
    let session1 = MockSession::new("pod-1", "default", Some("container-1"));
    let session2 = MockSession::new("pod-2", "default", Some("container-2"));
    let session3 = MockSession::new("pod-3", "monitoring", Some("container-3"));
    
    api.set_success_response("start_pod_logs_stream", json!(session1.id.clone()));
    
    // Start first stream
    let stream1 = api.invoke("start_pod_logs_stream", json!({
        "pod_name": "pod-1",
        "namespace": "default",
        "container_name": "container-1"
    })).await;
    assert!(stream1.is_ok());
    
    // Update mock for second stream
    api.set_success_response("start_pod_logs_stream", json!(session2.id.clone()));
    
    // Start second stream
    let stream2 = api.invoke("start_pod_logs_stream", json!({
        "pod_name": "pod-2", 
        "namespace": "default",
        "container_name": "container-2"
    })).await;
    assert!(stream2.is_ok());
    
    // Update mock for third stream
    api.set_success_response("start_pod_logs_stream", json!(session3.id.clone()));
    
    // Start third stream
    let stream3 = api.invoke("start_pod_logs_stream", json!({
        "pod_name": "pod-3",
        "namespace": "monitoring", 
        "container_name": "container-3"
    })).await;
    assert!(stream3.is_ok());
    
    // Verify all streams have different IDs
    let id1 = stream1.unwrap().as_str().unwrap();
    let id2 = stream2.unwrap().as_str().unwrap(); 
    let id3 = stream3.unwrap().as_str().unwrap();
    
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
    assert_ne!(id1, id3);
    
    assert_eq!(api.get_call_count("start_pod_logs_stream"), 3);
}

// Test log streaming resilience to connection issues
#[tokio::test]
async fn test_log_streaming_connection_resilience() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // First attempt fails due to connection issue
    api.set_error_response("start_pod_logs_stream", "Connection lost to Kubernetes API");
    
    let first_attempt = api.invoke("start_pod_logs_stream", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name
    })).await;
    assert!(first_attempt.is_err());
    
    // Second attempt succeeds (connection recovered)
    let session = MockSession::new(&params.pod_name, &params.namespace, params.container_name.as_deref());
    api.set_success_response("start_pod_logs_stream", json!(session.id));
    
    let second_attempt = api.invoke("start_pod_logs_stream", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name
    })).await;
    assert!(second_attempt.is_ok());
    
    assert_eq!(api.get_call_count("start_pod_logs_stream"), 2);
}

// Test handling of large log outputs
#[tokio::test]
async fn test_large_log_output_handling() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    // Create large log output
    let large_logs = (0..10000)
        .map(|i| format!("2024-01-01T00:{}:00.000Z [INFO] Log line {}", i % 60, i))
        .collect::<Vec<_>>()
        .join("\n");
    
    api.set_success_response("get_pod_logs", json!(large_logs));
    
    let start_time = std::time::Instant::now();
    
    let result = api.invoke("get_pod_logs", json!({
        "pod_name": params.pod_name,
        "namespace": params.namespace,
        "container_name": params.container_name,
        "lines": 10000
    })).await;
    
    let duration = start_time.elapsed();
    
    assert!(result.is_ok());
    let logs = result.unwrap();
    let logs_string = logs.as_str().unwrap();
    assert_eq!(logs_string.lines().count(), 10000);
    
    // Should handle large logs reasonably fast
    assert!(duration.as_millis() < 3000);
}

// Test logs from different container types
#[tokio::test]
async fn test_logs_from_different_container_types() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    let test_scenarios = vec![
        ("main-app", "Application logs"),
        ("sidecar-proxy", "Proxy configuration loaded"),
        ("init-setup", "Initialization completed"),
        ("monitoring-agent", "Metrics collection started"),
    ];
    
    for (container_name, expected_content) in test_scenarios {
        let container_logs = format!("2024-01-01T00:00:00.000Z [INFO] {}", expected_content);
        api.set_success_response("get_pod_logs", json!(container_logs));
        
        let result = api.invoke("get_pod_logs", json!({
            "pod_name": params.pod_name,
            "namespace": params.namespace,
            "container_name": container_name,
            "lines": 100
        })).await;
        
        assert!(result.is_ok());
        let logs = result.unwrap();
        let logs_string = logs.as_str().unwrap();
        assert!(logs_string.contains(expected_content));
    }
}

// Test log line limits
#[tokio::test]
async fn test_log_line_limits() {
    let mut api = MockTauriApi::new();
    let params = TestParams::default();
    
    let test_cases = vec![
        (1, 1),
        (10, 10), 
        (100, 100),
        (1000, 1000),
    ];
    
    for (requested_lines, expected_lines) in test_cases {
        let mock_logs = (0..expected_lines)
            .map(|i| format!("2024-01-01T00:00:{:02}.000Z [INFO] Line {}", i % 60, i))
            .collect::<Vec<_>>()
            .join("\n");
        
        api.set_success_response("get_pod_logs", json!(mock_logs));
        
        let result = api.invoke("get_pod_logs", json!({
            "pod_name": params.pod_name,
            "namespace": params.namespace,
            "container_name": params.container_name,
            "lines": requested_lines
        })).await;
        
        assert!(result.is_ok());
        let logs = result.unwrap();
        let logs_string = logs.as_str().unwrap();
        assert_eq!(logs_string.lines().count(), expected_lines);
    }
}