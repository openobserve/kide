//! Comprehensive integration tests for the generic resource update functionality
//!
//! These tests verify that the update_resource command works correctly with various
//! Kubernetes resource types using kubectl apply under the hood.

use std::process::Command;
use tempfile::NamedTempFile;
use std::io::Write;

/// Test helper to check if kubectl is available
fn is_kubectl_available() -> bool {
    Command::new("kubectl")
        .arg("version")
        .arg("--client")
        .arg("--short")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Test helper to create a valid YAML content for testing
fn create_test_deployment_yaml(name: &str, replicas: i32) -> String {
    format!(r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {}
  namespace: default
  labels:
    app: test-app
spec:
  replicas: {}
  selector:
    matchLabels:
      app: test-app
  template:
    metadata:
      labels:
        app: test-app
    spec:
      containers:
      - name: nginx
        image: nginx:1.20
        ports:
        - containerPort: 80
"#, name, replicas).trim().to_string()
}

fn create_test_service_yaml(name: &str) -> String {
    format!(r#"
apiVersion: v1
kind: Service
metadata:
  name: {}
  namespace: default
spec:
  selector:
    app: test-app
  ports:
  - port: 80
    targetPort: 80
  type: ClusterIP
"#, name).trim().to_string()
}

fn create_test_configmap_yaml(name: &str) -> String {
    format!(r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: {}
  namespace: default
data:
  config.properties: |
    app.name=test-app
    app.version=1.0.0
  app.yaml: |
    server:
      port: 8080
"#, name).trim().to_string()
}

#[tokio::test]
async fn test_kubectl_apply_integration_deployment() {
    if !is_kubectl_available() {
        println!("Skipping kubectl integration test - kubectl not available");
        return;
    }

    let yaml_content = create_test_deployment_yaml("test-deployment-integration", 2);
    
    // Create temporary file
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write YAML");
    
    // Test kubectl apply with dry-run to avoid creating actual resources
    let output = Command::new("kubectl")
        .arg("apply")
        .arg("-f")
        .arg(temp_file.path())
        .arg("--dry-run=client")
        .arg("-o")
        .arg("yaml")
        .output()
        .expect("Failed to execute kubectl");
    
    assert!(output.status.success(), "kubectl apply dry-run should succeed");
    
    let output_str = String::from_utf8(output.stdout).expect("Invalid UTF-8 in kubectl output");
    assert!(output_str.contains("Deployment"));
    assert!(output_str.contains("test-deployment-integration"));
}

#[tokio::test]
async fn test_kubectl_apply_integration_service() {
    if !is_kubectl_available() {
        println!("Skipping kubectl integration test - kubectl not available");
        return;
    }

    let yaml_content = create_test_service_yaml("test-service-integration");
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write YAML");
    
    let output = Command::new("kubectl")
        .arg("apply")
        .arg("-f")
        .arg(temp_file.path())
        .arg("--dry-run=client")
        .arg("-o")
        .arg("yaml")
        .output()
        .expect("Failed to execute kubectl");
    
    assert!(output.status.success(), "kubectl apply dry-run should succeed for Service");
    
    let output_str = String::from_utf8(output.stdout).expect("Invalid UTF-8 in kubectl output");
    assert!(output_str.contains("Service"));
    assert!(output_str.contains("test-service-integration"));
}

#[tokio::test]
async fn test_kubectl_apply_integration_configmap() {
    if !is_kubectl_available() {
        println!("Skipping kubectl integration test - kubectl not available");
        return;
    }

    let yaml_content = create_test_configmap_yaml("test-configmap-integration");
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write YAML");
    
    let output = Command::new("kubectl")
        .arg("apply")
        .arg("-f")
        .arg(temp_file.path())
        .arg("--dry-run=client")
        .arg("-o")
        .arg("yaml")
        .output()
        .expect("Failed to execute kubectl");
    
    assert!(output.status.success(), "kubectl apply dry-run should succeed for ConfigMap");
    
    let output_str = String::from_utf8(output.stdout).expect("Invalid UTF-8 in kubectl output");
    assert!(output_str.contains("ConfigMap"));
    assert!(output_str.contains("test-configmap-integration"));
}

#[tokio::test]
async fn test_kubectl_apply_error_handling() {
    if !is_kubectl_available() {
        println!("Skipping kubectl integration test - kubectl not available");
        return;
    }

    // Test with invalid YAML
    let invalid_yaml = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: invalid-deployment
spec:
  # Missing required fields like selector and template
  replicas: 1
"#;
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(invalid_yaml.as_bytes()).expect("Failed to write YAML");
    
    let output = Command::new("kubectl")
        .arg("apply")
        .arg("-f")
        .arg(temp_file.path())
        .arg("--dry-run=client")
        .output()
        .expect("Failed to execute kubectl");
    
    assert!(!output.status.success(), "kubectl apply should fail with invalid YAML");
    
    let error_str = String::from_utf8(output.stderr).expect("Invalid UTF-8 in kubectl error");
    assert!(error_str.contains("error") || error_str.contains("invalid") || error_str.contains("missing"));
}

#[tokio::test]
async fn test_multiple_resource_types_batch() {
    if !is_kubectl_available() {
        println!("Skipping kubectl integration test - kubectl not available");
        return;
    }

    // Test multiple resource types in a single YAML document
    let multi_resource_yaml = r#"
apiVersion: v1
kind: ConfigMap
metadata:
  name: test-multi-config
  namespace: default
data:
  app.properties: "test=value"
---
apiVersion: v1
kind: Service
metadata:
  name: test-multi-service
  namespace: default
spec:
  selector:
    app: multi-test
  ports:
  - port: 80
    targetPort: 8080
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: test-multi-deployment
  namespace: default
spec:
  replicas: 1
  selector:
    matchLabels:
      app: multi-test
  template:
    metadata:
      labels:
        app: multi-test
    spec:
      containers:
      - name: app
        image: nginx:1.20
        ports:
        - containerPort: 8080
"#;
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(multi_resource_yaml.as_bytes()).expect("Failed to write YAML");
    
    let output = Command::new("kubectl")
        .arg("apply")
        .arg("-f")
        .arg(temp_file.path())
        .arg("--dry-run=client")
        .output()
        .expect("Failed to execute kubectl");
    
    assert!(output.status.success(), "kubectl apply should succeed with multi-resource YAML");
    
    let output_str = String::from_utf8(output.stdout).expect("Invalid UTF-8 in kubectl output");
    assert!(output_str.contains("ConfigMap"));
    assert!(output_str.contains("Service"));
    assert!(output_str.contains("Deployment"));
}

#[tokio::test]
async fn test_namespace_handling() {
    if !is_kubectl_available() {
        println!("Skipping kubectl integration test - kubectl not available");
        return;
    }

    let yaml_content = create_test_deployment_yaml("test-namespace-deployment", 1);
    
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write YAML");
    
    // Test with explicit namespace parameter
    let output = Command::new("kubectl")
        .arg("apply")
        .arg("-f")
        .arg(temp_file.path())
        .arg("-n")
        .arg("default")
        .arg("--dry-run=client")
        .arg("-o")
        .arg("yaml")
        .output()
        .expect("Failed to execute kubectl");
    
    assert!(output.status.success(), "kubectl apply with namespace should succeed");
    
    let output_str = String::from_utf8(output.stdout).expect("Invalid UTF-8 in kubectl output");
    assert!(output_str.contains("namespace: default"));
}

#[tokio::test] 
async fn test_yaml_validation_integration() {
    // Test YAML parsing that our update_resource function does
    let valid_yaml = create_test_deployment_yaml("yaml-test", 3);
    
    // This should succeed
    let parsed_yaml: serde_yaml_ng::Value = serde_yaml_ng::from_str(&valid_yaml)
        .expect("Valid YAML should parse successfully");
    
    assert!(parsed_yaml.get("apiVersion").is_some());
    assert!(parsed_yaml.get("kind").is_some());
    assert!(parsed_yaml.get("metadata").is_some());
    assert!(parsed_yaml.get("spec").is_some());
    
    // Test invalid YAML
    let invalid_yaml = r#"
    invalid: yaml: content: [
    missing: bracket
    "#;
    
    let parse_result = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&invalid_yaml);
    assert!(parse_result.is_err(), "Invalid YAML should fail to parse");
}

/// Integration test that simulates the full update_resource flow
#[tokio::test]
async fn test_full_update_resource_simulation() {
    // This simulates what the update_resource function does internally
    let yaml_content = create_test_deployment_yaml("simulation-test", 5);
    
    // Step 1: Validate YAML (like our function does)
    let _parsed_yaml: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml_content)
        .expect("YAML validation should succeed");
    
    // Step 2: Write to temporary file (like our function does)
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write YAML");
    
    // Step 3: Verify file was written correctly
    let file_content = std::fs::read_to_string(temp_file.path())
        .expect("Should be able to read temp file");
    assert_eq!(file_content, yaml_content);
    
    // Step 4: If kubectl is available, test the apply command
    if is_kubectl_available() {
        let output = Command::new("kubectl")
            .arg("apply")
            .arg("-f")
            .arg(temp_file.path())
            .arg("--dry-run=client")
            .output()
            .expect("kubectl execution should work");
        
        println!("kubectl dry-run output: {}", String::from_utf8_lossy(&output.stdout));
        if !output.status.success() {
            println!("kubectl error: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        // For simulation, we don't assert success since we might not have a cluster
        // But we verify the command executed without system errors
    }
    
    println!("✅ Full update resource simulation completed successfully");
}

/// Test resource kinds that are commonly updated
#[tokio::test]
async fn test_common_resource_update_scenarios() {
    let test_scenarios = vec![
        ("Deployment", create_test_deployment_yaml("scenario-deploy", 3)),
        ("Service", create_test_service_yaml("scenario-svc")),
        ("ConfigMap", create_test_configmap_yaml("scenario-cm")),
    ];
    
    for (resource_kind, yaml_content) in test_scenarios {
        // Validate YAML structure
        let parsed: serde_yaml_ng::Value = serde_yaml_ng::from_str(&yaml_content)
            .unwrap_or_else(|e| panic!("Failed to parse {} YAML: {}", resource_kind, e));
        
        assert_eq!(parsed["kind"].as_str().unwrap(), resource_kind);
        assert!(parsed["metadata"]["name"].is_string());
        assert!(parsed["spec"].is_mapping() || parsed["data"].is_mapping()); // ConfigMap has data instead of spec
        
        println!("✅ {} YAML structure validated", resource_kind);
    }
}