/**
 * API Integration Test Helpers
 * 
 * Common utilities, mocks, and test data for API endpoint testing.
 */

use std::collections::HashMap;
use serde_json::{json, Value};

// Mock Tauri invoke function for testing
pub struct MockTauriApi {
    pub responses: HashMap<String, Result<Value, String>>,
    pub call_counts: HashMap<String, usize>,
}

impl MockTauriApi {
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            call_counts: HashMap::new(),
        }
    }

    pub fn set_response(&mut self, command: &str, response: Result<Value, String>) {
        self.responses.insert(command.to_string(), response);
    }

    pub fn set_success_response(&mut self, command: &str, data: Value) {
        self.responses.insert(command.to_string(), Ok(data));
    }

    pub fn set_error_response(&mut self, command: &str, error: &str) {
        self.responses.insert(command.to_string(), Err(error.to_string()));
    }

    pub async fn invoke(&mut self, command: &str, _args: Value) -> Result<Value, String> {
        // Track call count
        let count = self.call_counts.entry(command.to_string()).or_insert(0);
        *count += 1;

        // Return mock response
        match self.responses.get(command) {
            Some(response) => response.clone(),
            None => Err(format!("No mock response set for command: {}", command)),
        }
    }

    pub fn get_call_count(&self, command: &str) -> usize {
        self.call_counts.get(command).copied().unwrap_or(0)
    }

    pub fn was_called(&self, command: &str) -> bool {
        self.get_call_count(command) > 0
    }
}

// Mock Kubernetes contexts
pub fn mock_k8s_contexts() -> Value {
    json!([
        {
            "name": "minikube",
            "cluster": "minikube",
            "user": "minikube",
            "namespace": "default"
        },
        {
            "name": "docker-desktop",
            "cluster": "docker-desktop",
            "user": "docker-desktop",
            "namespace": "default"
        },
        {
            "name": "production-cluster",
            "cluster": "prod-cluster",
            "user": "prod-user",
            "namespace": "kube-system"
        }
    ])
}

// Mock resource categories
pub fn mock_resource_categories() -> Value {
    json!([
        {
            "name": "Workloads",
            "resources": [
                {
                    "name": "Pods",
                    "kind": "Pod",
                    "apiVersion": "v1",
                    "namespaced": true,
                    "shortNames": ["po"]
                },
                {
                    "name": "Deployments",
                    "kind": "Deployment",
                    "apiVersion": "apps/v1",
                    "namespaced": true,
                    "shortNames": ["deploy"]
                }
            ]
        },
        {
            "name": "Services",
            "resources": [
                {
                    "name": "Services",
                    "kind": "Service",
                    "apiVersion": "v1",
                    "namespaced": true,
                    "shortNames": ["svc"]
                }
            ]
        }
    ])
}

// Mock namespaces
pub fn mock_namespaces() -> Value {
    json!([
        "default",
        "kube-system",
        "kube-public",
        "kube-node-lease",
        "my-app",
        "monitoring"
    ])
}

// Mock pod data
pub fn mock_pod() -> Value {
    json!({
        "metadata": {
            "name": "test-pod",
            "namespace": "default",
            "uid": "12345-67890-abcdef",
            "creationTimestamp": "2024-01-01T00:00:00Z",
            "labels": {
                "app": "test-app",
                "version": "v1.0.0"
            }
        },
        "spec": {
            "containers": [
                {
                    "name": "main-container",
                    "image": "nginx:latest",
                    "ports": [
                        {
                            "containerPort": 80,
                            "protocol": "TCP"
                        }
                    ]
                }
            ],
            "nodeName": "minikube"
        },
        "status": {
            "phase": "Running",
            "podIP": "10.244.0.1",
            "containerStatuses": [
                {
                    "name": "main-container",
                    "ready": true,
                    "restartCount": 0,
                    "state": {
                        "running": {
                            "startedAt": "2024-01-01T00:01:00Z"
                        }
                    }
                }
            ]
        }
    })
}

// Mock deployment data
pub fn mock_deployment() -> Value {
    json!({
        "metadata": {
            "name": "test-deployment",
            "namespace": "default",
            "uid": "deploy-12345",
            "creationTimestamp": "2024-01-01T00:00:00Z"
        },
        "spec": {
            "replicas": 3,
            "selector": {
                "matchLabels": {
                    "app": "test-app"
                }
            },
            "template": {
                "metadata": {
                    "labels": {
                        "app": "test-app"
                    }
                },
                "spec": {
                    "containers": [
                        {
                            "name": "app",
                            "image": "nginx:latest"
                        }
                    ]
                }
            }
        },
        "status": {
            "readyReplicas": 3,
            "replicas": 3,
            "availableReplicas": 3
        }
    })
}

// Mock events data
pub fn mock_events() -> Value {
    json!([
        {
            "metadata": {
                "name": "event-1",
                "namespace": "default",
                "creationTimestamp": "2024-01-01T00:00:00Z"
            },
            "involvedObject": {
                "kind": "Pod",
                "name": "test-pod",
                "namespace": "default"
            },
            "reason": "Scheduled",
            "message": "Successfully assigned default/test-pod to minikube",
            "type": "Normal",
            "firstTimestamp": "2024-01-01T00:00:00Z",
            "lastTimestamp": "2024-01-01T00:00:00Z",
            "count": 1
        }
    ])
}

// Mock pod logs
pub fn mock_pod_logs() -> String {
    r#"2024-01-01T00:00:00.000Z [INFO] Starting application
2024-01-01T00:00:01.000Z [INFO] Server listening on port 80
2024-01-01T00:00:02.000Z [INFO] Health check endpoint available at /health
2024-01-01T00:00:03.000Z [DEBUG] Processing incoming request
2024-01-01T00:00:04.000Z [INFO] Request processed successfully"#.to_string()
}

// Test error scenarios
pub fn mock_connection_error() -> String {
    "Failed to connect to Kubernetes cluster: connection refused".to_string()
}

pub fn mock_auth_error() -> String {
    "Authentication failed: invalid credentials".to_string()
}

pub fn mock_not_found_error() -> String {
    "Resource not found".to_string()
}

pub fn mock_validation_error() -> String {
    "Invalid resource specification".to_string()
}

// Helper for creating test scenarios
pub struct TestScenario {
    pub name: &'static str,
    pub description: &'static str,
    pub should_succeed: bool,
    pub expected_calls: Vec<&'static str>,
}

impl TestScenario {
    pub fn new(name: &'static str, description: &'static str, should_succeed: bool) -> Self {
        Self {
            name,
            description,
            should_succeed,
            expected_calls: Vec::new(),
        }
    }

    pub fn with_calls(mut self, calls: Vec<&'static str>) -> Self {
        self.expected_calls = calls;
        self
    }
}

// Common test parameters
pub struct TestParams {
    pub pod_name: String,
    pub namespace: String,
    pub resource_kind: String,
    pub context_name: String,
    pub container_name: Option<String>,
}

impl Default for TestParams {
    fn default() -> Self {
        Self {
            pod_name: "test-pod".to_string(),
            namespace: "default".to_string(),
            resource_kind: "Pod".to_string(),
            context_name: "minikube".to_string(),
            container_name: Some("main-container".to_string()),
        }
    }
}

// Session management for shell and logs
pub struct MockSession {
    pub id: String,
    pub active: bool,
    pub pod_name: String,
    pub namespace: String,
    pub container_name: Option<String>,
}

impl MockSession {
    pub fn new(pod_name: &str, namespace: &str, container_name: Option<&str>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            active: true,
            pod_name: pod_name.to_string(),
            namespace: namespace.to_string(),
            container_name: container_name.map(|s| s.to_string()),
        }
    }
}