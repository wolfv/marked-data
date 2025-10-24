//! Example demonstrating flow style control using #[serde(serialize_with)]
//!
//! This example shows the recommended approach for per-field flow style control
//! using serde's built-in `serialize_with` attribute.
//!
//! Works with HashMap, BTreeMap, and any other map-like collections.

use marked_yaml::{as_flow_mapping, as_flow_sequence, to_yaml_string};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

/// Example 1: Flow style for specific sequence fields
#[derive(Serialize)]
struct WebServer {
    name: String,

    // This field will use flow style: [80, 443, 8080]
    #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
    ports: Vec<i32>,

    // This field will use block style (default)
    servers: Vec<String>,
}

fn example_mixed_sequence_styles() {
    let config = WebServer {
        name: "MyApp".to_string(),
        ports: vec![80, 443, 8080],
        servers: vec!["server1".to_string(), "server2".to_string()],
    };

    println!("=== Mixed Sequence Styles ===\n");
    let yaml = to_yaml_string(&config).unwrap();
    println!("{}", yaml);
    println!("Note: 'ports' uses flow style [80, 443, 8080]");
    println!("      'servers' uses block style (- server1, - server2)\n");
}

/// Example 2: Flow style for mapping fields
#[derive(Serialize)]
struct Application {
    name: String,
    version: i32,

    // This field will use flow style: {env: prod, app: myapp}
    #[serde(serialize_with = "marked_yaml::as_flow_mapping")]
    labels: HashMap<String, String>,

    // This field will use block style (default)
    config: HashMap<String, String>,
}

fn example_mixed_mapping_styles() {
    let mut labels = HashMap::new();
    labels.insert("env".to_string(), "prod".to_string());
    labels.insert("app".to_string(), "myapp".to_string());

    let mut config = HashMap::new();
    config.insert("timeout".to_string(), "30s".to_string());
    config.insert("retries".to_string(), "3".to_string());

    let app = Application {
        name: "WebApp".to_string(),
        version: 1,
        labels,
        config,
    };

    println!("=== Mixed Mapping Styles ===\n");
    let yaml = to_yaml_string(&app).unwrap();
    println!("{}", yaml);
    println!("Note: 'labels' uses flow style {{env: prod, app: myapp}}");
    println!("      'config' uses block style (key: value on separate lines)\n");
}

/// Example 3: Nested structures with flow annotations
#[derive(Serialize)]
struct Address {
    street: String,
    city: String,
    zip: String,
}

#[derive(Serialize)]
struct Person {
    name: String,
    age: i32,

    // Array of years - the array itself uses flow style
    #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
    important_years: Vec<i32>,

    // Regular block style for addresses
    addresses: Vec<Address>,
}

fn example_with_nested_structures() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        important_years: vec![2020, 2021, 2022],
        addresses: vec![
            Address {
                street: "123 Main St".to_string(),
                city: "Boston".to_string(),
                zip: "02101".to_string(),
            },
            Address {
                street: "456 Oak Ave".to_string(),
                city: "Cambridge".to_string(),
                zip: "02139".to_string(),
            },
        ],
    };

    println!("=== Nested Structures with Selective Flow Style ===\n");
    let yaml = to_yaml_string(&person).unwrap();
    println!("{}", yaml);
    println!("Note: 'important_years' uses flow style [2020, 2021, 2022]");
    println!("      'addresses' uses block style for better readability\n");
}

/// Example 4: Complex configuration with multiple flow-styled fields
/// Demonstrates that as_flow_mapping works with both HashMap and BTreeMap
#[derive(Serialize)]
struct K8sDeployment {
    api_version: String,
    kind: String,

    // Works with HashMap
    #[serde(serialize_with = "marked_yaml::as_flow_mapping")]
    metadata: HashMap<String, String>,

    #[serde(rename = "spec")]
    specification: DeploymentSpec,
}

#[derive(Serialize)]
struct DeploymentSpec {
    replicas: i32,

    #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
    ports: Vec<i32>,

    // Works with BTreeMap too
    env_vars: BTreeMap<String, String>,
}

fn example_kubernetes_style() {
    let mut metadata = HashMap::new();
    metadata.insert("name".to_string(), "my-app".to_string());
    metadata.insert("namespace".to_string(), "default".to_string());

    let mut env_vars = BTreeMap::new();
    env_vars.insert("LOG_LEVEL".to_string(), "info".to_string());
    env_vars.insert("DATABASE_URL".to_string(), "postgres://...".to_string());

    let deployment = K8sDeployment {
        api_version: "apps/v1".to_string(),
        kind: "Deployment".to_string(),
        metadata,
        specification: DeploymentSpec {
            replicas: 3,
            ports: vec![8080, 8443],
            env_vars,
        },
    };

    println!("=== Kubernetes-Style Configuration ===\n");
    let yaml = to_yaml_string(&deployment).unwrap();
    println!("{}", yaml);
    println!("Note: Compact flow style for metadata and ports");
    println!("      Block style for verbose env_vars\n");
}

fn main() {
    example_mixed_sequence_styles();
    println!("{}\n", "=".repeat(60));

    example_mixed_mapping_styles();
    println!("{}\n", "=".repeat(60));

    example_with_nested_structures();
    println!("{}\n", "=".repeat(60));

    example_kubernetes_style();
    println!("{}\n", "=".repeat(60));

    println!("=== Summary ===\n");
    println!("Per-field flow style control is now fully supported!");
    println!("\nUse #[serde(serialize_with = \"marked_yaml::as_flow_sequence\")]");
    println!("    for inline arrays: [1, 2, 3]");
    println!("\nUse #[serde(serialize_with = \"marked_yaml::as_flow_mapping\")]");
    println!("    for inline objects: {{key: value}}");
    println!("\nMix and match to create readable, compact YAML output!");
}
