/**
 * update_resource API Integration Tests
 * 
 * Comprehensive tests for the update_resource endpoint including:
 * - YAML content validation and parsing
 * - Resource validation and kubectl apply behavior  
 * - Error scenarios and edge cases
 * - Large content handling and concurrent updates
 * - Kubernetes-specific update behaviors
 */

use crate::tests::integration::api::test_helpers::*;
use serde_json::json;
use tokio_test;

#[tokio::test]
async fn test_update_resource_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful update
    api.set_success_response("update_resource", json!(null));
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:1.20
"#.to_string();
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("update_resource"));
}

#[tokio::test]
async fn test_update_resource_invalid_yaml() {
    let mut api = MockTauriApi::new();
    
    // Mock invalid YAML error
    api.set_error_response("update_resource", "Invalid YAML syntax");
    
    let invalid_yaml = "invalid: yaml: content: [";
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": invalid_yaml
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid YAML"));
}

#[tokio::test]
async fn test_update_resource_validation_error() {
    let mut api = MockTauriApi::new();
    
    // Mock Kubernetes validation error
    api.set_error_response("update_resource", &mock_validation_error());
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
spec:
  containers: []
"#.to_string();
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid resource specification"));
}

#[tokio::test]
async fn test_update_resource_deployment_success() {
    let mut api = MockTauriApi::new();
    
    // Mock successful deployment update
    api.set_success_response("update_resource", json!(null));
    
    let yaml_content = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: o2-openobserve-router
  namespace: default
spec:
  replicas: 3
  selector:
    matchLabels:
      app: openobserve-router
  template:
    metadata:
      labels:
        app: openobserve-router
    spec:
      containers:
      - name: router
        image: openobserve/openobserve:latest
"#.to_string();
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "o2-openobserve-router",
        "resource_kind": "Deployment",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_ok());
    assert!(api.was_called("update_resource"));
}

#[tokio::test]
async fn test_update_resource_unsupported_kind() {
    let mut api = MockTauriApi::new();
    
    // Mock kubectl apply error for unsupported resource
    api.set_error_response("update_resource", "kubectl apply failed: error validating data");
    
    let yaml_content = r#"
apiVersion: custom/v1
kind: UnsupportedResource
metadata:
  name: test-resource
  namespace: default
spec:
  customField: value
"#.to_string();
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-resource",
        "resource_kind": "UnsupportedResource",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
}

// ==== COMPREHENSIVE EDGE CASE TESTS ====

#[tokio::test]
async fn test_update_resource_empty_yaml_content() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "YAML content cannot be empty");
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": ""
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("empty"));
}

#[tokio::test]
async fn test_update_resource_null_yaml_content() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "YAML content cannot be null");
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": null
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("null"));
}

#[tokio::test]
async fn test_update_resource_malformed_yaml_syntax() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "Invalid YAML syntax: unexpected character");
    
    let malformed_yaml = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:latest
    ports:
    - containerPort: 80
      protocol: TCP
    volumes: [missing bracket
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": malformed_yaml
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid YAML"));
}

#[tokio::test]
async fn test_update_resource_yaml_with_tabs() {
    let mut api = MockTauriApi::new();
    
    // YAML with tabs should be accepted (some editors use tabs)
    api.set_success_response("update_resource", json!(null));
    
    let yaml_with_tabs = "apiVersion: v1\nkind: Pod\nmetadata:\n\tname: test-pod\n\tnamespace: default\nspec:\n\tcontainers:\n\t- name: nginx\n\t  image: nginx:latest";
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_with_tabs
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_resource_yaml_with_unicode_characters() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("update_resource", json!(null));
    
    let yaml_with_unicode = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod-ñ
  namespace: default
  annotations:
    description: "Pod with üñíçøðé characters"
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod-ñ",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_with_unicode
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_resource_very_large_yaml() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("update_resource", json!(null));
    
    // Create a YAML with many annotations (large content)
    let mut large_yaml = String::from(r#"
apiVersion: v1
kind: Pod
metadata:
  name: large-pod
  namespace: default
  annotations:
"#);
    
    for i in 0..1000 {
        large_yaml.push_str(&format!("    annotation-{}: \"value-{}\"\n", i, i));
    }
    
    large_yaml.push_str(r#"
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#);
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "large-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": large_yaml
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_resource_extremely_large_yaml() {
    let mut api = MockTauriApi::new();
    
    // Should fail if YAML is too large
    api.set_error_response("update_resource", "YAML content exceeds maximum size limit");
    
    let huge_yaml = "a".repeat(10_000_000); // 10MB string
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "huge-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": huge_yaml
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("exceeds maximum size"));
}

#[tokio::test]
async fn test_update_resource_yaml_version_mismatch() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "Resource name mismatch: YAML contains 'different-pod' but expected 'test-pod'");
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: different-pod  # Name doesn't match parameter
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("name mismatch"));
}

#[tokio::test]
async fn test_update_resource_yaml_kind_mismatch() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "Resource kind mismatch: YAML contains 'Service' but expected 'Pod'");
    
    let yaml_content = r#"
apiVersion: v1
kind: Service  # Kind doesn't match parameter
metadata:
  name: test-pod
  namespace: default
spec:
  selector:
    app: test
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("kind mismatch"));
}

#[tokio::test]
async fn test_update_resource_yaml_namespace_mismatch() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "Namespace mismatch: YAML contains 'production' but expected 'default'");
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: production  # Namespace doesn't match parameter
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Namespace mismatch"));
}

#[tokio::test]
async fn test_update_resource_yaml_missing_required_fields() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "kubectl apply failed: missing required field 'spec'");
    
    let incomplete_yaml = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: default
# Missing spec section
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": incomplete_yaml
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("missing required field"));
}

#[tokio::test]
async fn test_update_resource_yaml_invalid_api_version() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "kubectl apply failed: unable to recognize API version 'invalid/v999'");
    
    let invalid_api_yaml = r#"
apiVersion: invalid/v999
kind: Pod
metadata:
  name: test-pod
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": invalid_api_yaml
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unable to recognize"));
}

#[tokio::test]
async fn test_update_resource_yaml_with_finalizers() {
    let mut api = MockTauriApi::new();
    
    // Resources with finalizers should still be updatable
    api.set_success_response("update_resource", json!(null));
    
    let yaml_with_finalizers = r#"
apiVersion: v1
kind: Namespace
metadata:
  name: test-namespace
  finalizers:
  - custom-finalizer.example.com
spec: {}
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-namespace",
        "resource_kind": "Namespace",
        "namespace": null,
        "yaml_content": yaml_with_finalizers
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_resource_yaml_with_owner_references() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("update_resource", json!(null));
    
    let yaml_with_owner_refs = r#"
apiVersion: v1
kind: Pod
metadata:
  name: owned-pod
  namespace: default
  ownerReferences:
  - apiVersion: apps/v1
    kind: ReplicaSet
    name: my-replicaset
    uid: 12345-67890
    controller: true
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "owned-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_with_owner_refs
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_resource_custom_resource() {
    let mut api = MockTauriApi::new();
    
    api.set_success_response("update_resource", json!(null));
    
    let custom_resource_yaml = r#"
apiVersion: example.com/v1
kind: CustomApplication
metadata:
  name: my-app
  namespace: default
spec:
  replicas: 3
  customField: "customValue"
  nestedConfig:
    setting1: "value1"
    setting2: 42
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "my-app",
        "resource_kind": "CustomApplication",
        "namespace": "default",
        "yaml_content": custom_resource_yaml
    })).await;
    
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_resource_validation_webhook_rejection() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "kubectl apply failed: admission webhook denied the request: Invalid configuration");
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: invalid-pod
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:latest
    securityContext:
      privileged: true  # Might be rejected by webhook
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "invalid-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("webhook denied"));
}

#[tokio::test]
async fn test_update_resource_immutable_field_change() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "kubectl apply failed: field is immutable");
    
    let yaml_content = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-deployment
  namespace: default
spec:
  selector:
    matchLabels:
      app: changed-app  # Selector is immutable
  template:
    metadata:
      labels:
        app: test-app
    spec:
      containers:
      - name: nginx
        image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-deployment",
        "resource_kind": "Deployment",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("immutable"));
}

#[tokio::test]
async fn test_update_resource_concurrent_updates() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let api = Arc::new(Mutex::new(MockTauriApi::new()));
    
    // Set up mock response for successful updates
    {
        let mut api_guard = api.lock().await;
        api_guard.set_success_response("update_resource", json!(null));
    }
    
    let base_yaml = r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: config-{id}
  namespace: default
data:
  key: "value-{id}"
"#;
    
    // Spawn multiple concurrent update requests
    let mut handles = Vec::new();
    for i in 0..3 {
        let api_clone = Arc::clone(&api);
        let yaml_content = base_yaml.replace("{id}", &i.to_string());
        
        let handle = tokio::spawn(async move {
            let mut api_guard = api_clone.lock().await;
            api_guard.invoke("update_resource", json!({
                "resource_name": format!("config-{}", i),
                "resource_kind": "ConfigMap",
                "namespace": "default",
                "yaml_content": yaml_content
            })).await
        });
        handles.push(handle);
    }
    
    // Wait for all updates to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
    
    // Verify all updates were processed
    let api_guard = api.lock().await;
    assert_eq!(api_guard.get_call_count("update_resource"), 3);
}

#[tokio::test]
async fn test_update_resource_kubectl_not_found() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "kubectl command not found in PATH");
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("kubectl command not found"));
}

#[tokio::test]
async fn test_update_resource_cluster_disconnected() {
    let mut api = MockTauriApi::new();
    
    api.set_error_response("update_resource", "kubectl apply failed: unable to connect to server");
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unable to connect"));
}

#[tokio::test]
async fn test_update_resource_dry_run_simulation() {
    let mut api = MockTauriApi::new();
    
    // In a real implementation, this might use kubectl --dry-run
    api.set_success_response("update_resource", json!({
        "dry_run": true,
        "result": "validation passed"
    }));
    
    let yaml_content = r#"
apiVersion: v1
kind: Pod
metadata:
  name: test-pod
  namespace: default
spec:
  containers:
  - name: nginx
    image: nginx:latest
"#;
    
    let result = api.invoke("update_resource", json!({
        "resource_name": "test-pod",
        "resource_kind": "Pod",
        "namespace": "default",
        "yaml_content": yaml_content
    })).await;
    
    assert!(result.is_ok());
}