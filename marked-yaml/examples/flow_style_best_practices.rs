//! Best practices for flow style configuration in marked_yaml
//!
//! This example demonstrates the current recommended approaches for
//! controlling YAML output formatting.

use marked_yaml::{to_yaml_string, to_yaml_string_with_options, SerializerOptions};
use serde::Serialize;
use std::collections::HashMap;

/// Example 1: Simple configuration with arrays that benefit from flow style
#[derive(Serialize)]
struct ServerConfig {
    name: String,
    host: String,
    ports: Vec<u16>,
    allowed_ips: Vec<String>,
}

fn example_server_config() {
    println!("=== Example 1: Server Configuration ===\n");

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

    println!("Block style (readable but verbose):");
    println!("{}", to_yaml_string(&config).unwrap());

    println!("Flow style for arrays (compact for simple lists):");
    let mut opts = SerializerOptions::default();
    opts.flow_sequences = true;
    println!("{}", to_yaml_string_with_options(&config, &opts).unwrap());
}

/// Example 2: When to use block vs flow style
#[derive(Serialize)]
struct AppConfig {
    version: i32,
    // Short, simple arrays → flow style is good
    feature_flags: Vec<String>,
    // Nested objects → block style is better
    database: DatabaseConfig,
    // Many items → block style is more readable
    users: Vec<User>,
}

#[derive(Serialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
}

#[derive(Serialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

fn example_mixed_content() {
    println!("\n=== Example 2: Mixed Content ===\n");

    let config = AppConfig {
        version: 1,
        feature_flags: vec!["beta".to_string(), "experimental".to_string()],
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            name: "myapp".to_string(),
        },
        users: vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ],
    };

    println!("Default block style (best for complex nested data):");
    println!("{}", to_yaml_string(&config).unwrap());

    println!("\nWith flow sequences (good for simple arrays):");
    let mut opts = SerializerOptions::default();
    opts.flow_sequences = true;
    println!("{}", to_yaml_string_with_options(&config, &opts).unwrap());

    println!("\nFull flow style (very compact but less readable):");
    let mut opts = SerializerOptions::default();
    opts.flow_sequences = true;
    opts.flow_mappings = true;
    println!("{}", to_yaml_string_with_options(&config, &opts).unwrap());
}

/// Example 3: Kubernetes-style configuration
/// (Common use case: inline for simple values, block for complex ones)
#[derive(Serialize)]
struct KubernetesDeployment {
    api_version: String,
    kind: String,
    metadata: Metadata,
    spec: DeploymentSpec,
}

#[derive(Serialize)]
struct Metadata {
    name: String,
    labels: HashMap<String, String>,
}

#[derive(Serialize)]
struct DeploymentSpec {
    replicas: i32,
    selector: Selector,
    template: PodTemplate,
}

#[derive(Serialize)]
struct Selector {
    match_labels: HashMap<String, String>,
}

#[derive(Serialize)]
struct PodTemplate {
    metadata: Metadata,
    spec: PodSpec,
}

#[derive(Serialize)]
struct PodSpec {
    containers: Vec<Container>,
}

#[derive(Serialize)]
struct Container {
    name: String,
    image: String,
    ports: Vec<ContainerPort>,
    env: Vec<EnvVar>,
}

#[derive(Serialize)]
struct ContainerPort {
    container_port: i32,
}

#[derive(Serialize)]
struct EnvVar {
    name: String,
    value: String,
}

fn example_kubernetes_style() {
    println!("\n=== Example 3: Kubernetes-Style Configuration ===\n");

    let mut labels = HashMap::new();
    labels.insert("app".to_string(), "myapp".to_string());
    labels.insert("env".to_string(), "production".to_string());

    let deployment = KubernetesDeployment {
        api_version: "apps/v1".to_string(),
        kind: "Deployment".to_string(),
        metadata: Metadata {
            name: "myapp-deployment".to_string(),
            labels: labels.clone(),
        },
        spec: DeploymentSpec {
            replicas: 3,
            selector: Selector {
                match_labels: labels.clone(),
            },
            template: PodTemplate {
                metadata: Metadata {
                    name: "myapp-pod".to_string(),
                    labels: labels.clone(),
                },
                spec: PodSpec {
                    containers: vec![Container {
                        name: "myapp".to_string(),
                        image: "myapp:1.0.0".to_string(),
                        ports: vec![ContainerPort { container_port: 8080 }],
                        env: vec![
                            EnvVar {
                                name: "ENV".to_string(),
                                value: "production".to_string(),
                            },
                            EnvVar {
                                name: "DEBUG".to_string(),
                                value: "false".to_string(),
                            },
                        ],
                    }],
                },
            },
        },
    };

    println!("Standard block style (most readable for complex structures):");
    println!("{}", to_yaml_string(&deployment).unwrap());

    println!("\nWith flow sequences (compact for simple arrays like ports/env):");
    let mut opts = SerializerOptions::default();
    opts.flow_sequences = true;
    println!("{}", to_yaml_string_with_options(&deployment, &opts).unwrap());
}

/// Example 4: Recommendations summary
fn print_recommendations() {
    println!("\n=== Flow Style Recommendations ===\n");
    println!("✓ USE FLOW STYLE FOR:");
    println!("  • Short, simple arrays of primitives");
    println!("  • Configuration lists (ports, IPs, feature flags)");
    println!("  • Coordinate pairs or tuples");
    println!("  • Small key-value pairs\n");

    println!("✗ AVOID FLOW STYLE FOR:");
    println!("  • Complex nested structures");
    println!("  • Arrays with many items (>5)");
    println!("  • Objects with many fields");
    println!("  • Data that changes frequently (harder to diff)\n");

    println!("CURRENT APPROACH:");
    println!("  1. Start with default block style");
    println!("  2. Enable flow_sequences for simple arrays");
    println!("  3. Only use flow_mappings for very compact output needs\n");

    println!("FUTURE (when implemented):");
    println!("  • Use path-based config: flow_paths: [\"ports[]\", \"env[]\"]");
    println!("  • Use attributes: #[marked_yaml(flow)] on specific fields");
}

fn main() {
    example_server_config();
    example_mixed_content();
    example_kubernetes_style();
    print_recommendations();

    println!("\n=== Quick Reference ===\n");
    println!("Default (block):       to_yaml_string(&data)");
    println!("Flow sequences:        opts.flow_sequences = true");
    println!("Flow mappings:         opts.flow_mappings = true");
    println!("Both:                  opts.flow_sequences = true; opts.flow_mappings = true\n");

    println!("For more details, see:");
    println!("  • SERIALIZATION.md - Complete serialization guide");
    println!("  • FLOW_STYLE_GUIDE.md - Flow style configuration");
    println!("  • examples/serialization.rs - Basic examples\n");
}
