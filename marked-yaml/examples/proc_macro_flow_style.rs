//! Example demonstrating #[marked_yaml(flow)] proc macro attribute
//!
//! This example shows how to use the proc macro to control flow style
//! on a per-field basis.

use marked_yaml::to_yaml_string;

// Note: This uses the proc macro derive instead of serde::Serialize
use marked_yaml_derive::Serialize;

/// Example configuration with flow-style annotations
#[derive(Serialize)]
struct ServerConfig {
    /// Regular field - uses block style
    name: String,

    /// Regular field - uses block style
    host: String,

    /// Flow style for ports - will serialize as [80, 443, 8080]
    #[marked_yaml(flow)]
    ports: Vec<u16>,

    /// Flow style for IPs - will serialize as [10.0.0.1, 10.0.0.2]
    #[marked_yaml(flow)]
    allowed_ips: Vec<String>,
}

#[derive(Serialize)]
struct KubernetesConfig {
    api_version: String,
    kind: String,

    /// Flow style for labels
    #[marked_yaml(flow)]
    labels: std::collections::HashMap<String, String>,

    /// Regular nested structure
    spec: PodSpec,
}

#[derive(Serialize)]
struct PodSpec {
    replicas: i32,

    /// Flow style for container ports
    #[marked_yaml(flow)]
    ports: Vec<i32>,

    /// Flow style for environment variables
    #[marked_yaml(flow)]
    env: Vec<String>,
}

fn main() {
    println!("=== Example with #[marked_yaml(flow)] Attribute ===\n");

    let config = ServerConfig {
        name: "web-server-1".to_string(),
        host: "192.168.1.100".to_string(),
        ports: vec![80, 443, 8080],
        allowed_ips: vec![
            "10.0.0.1".to_string(),
            "10.0.0.2".to_string(),
            "10.0.0.3".to_string(),
        ],
    };

    let yaml = to_yaml_string(&config).unwrap();
    println!("{}", yaml);

    println!("\n=== Kubernetes-Style Example ===\n");

    let mut labels = std::collections::HashMap::new();
    labels.insert("app".to_string(), "myapp".to_string());
    labels.insert("env".to_string(), "prod".to_string());

    let k8s_config = KubernetesConfig {
        api_version: "v1".to_string(),
        kind: "Pod".to_string(),
        labels,
        spec: PodSpec {
            replicas: 3,
            ports: vec![8080, 8443],
            env: vec!["PROD".to_string(), "DEBUG=false".to_string()],
        },
    };

    let yaml = to_yaml_string(&k8s_config).unwrap();
    println!("{}", yaml);

    println!("\n=== Benefits ===");
    println!("✓ Per-field flow style control");
    println!("✓ Clean, declarative syntax");
    println!("✓ No need for global options");
    println!("✓ Works with standard serde ecosystem");
}
