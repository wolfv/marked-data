//! Test for global flow style options (clean, no thread-locals!)

use marked_yaml::{to_yaml_string_with_options, SerializerOptions};
use serde::Serialize;

#[derive(Serialize)]
struct BasicConfig {
    name: String,
    ports: Vec<i32>,
}

#[test]
fn test_global_flow_sequences() {
    let config = BasicConfig {
        name: "test".to_string(),
        ports: vec![80, 443, 8080],
    };

    let yaml =
        to_yaml_string_with_options(&config, &SerializerOptions::with_flow_sequences()).unwrap();

    println!("Generated YAML:\n{}", yaml);

    // Should have name field
    assert!(yaml.contains("name:"));
    assert!(yaml.contains("test"));

    // Should have ports field in flow style
    assert!(yaml.contains("ports:"));
    assert!(
        yaml.contains("[80, 443, 8080]"),
        "Expected flow style for ports, got:\n{}",
        yaml
    );
}

#[derive(Serialize)]
struct NestedConfig {
    version: i32,
    features: Vec<String>,
    database: DatabaseConfig,
}

#[derive(Serialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

#[test]
fn test_flow_sequences_nested() {
    let config = NestedConfig {
        version: 1,
        features: vec!["beta".to_string(), "experimental".to_string()],
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
        },
    };

    let yaml =
        to_yaml_string_with_options(&config, &SerializerOptions::with_flow_sequences()).unwrap();

    println!("Generated YAML:\n{}", yaml);

    // Should serialize correctly
    assert!(yaml.contains("version:"));
    assert!(yaml.contains("features:"));
    assert!(yaml.contains("database:"));

    // Features should be in flow style
    assert!(
        yaml.contains("[") && (yaml.contains("beta") || yaml.contains("[beta")),
        "Expected flow style for features, got:\n{}",
        yaml
    );
}

use std::collections::HashMap;

#[derive(Serialize)]
struct ConfigWithMap {
    name: String,
    labels: HashMap<String, String>,
}

#[test]
fn test_flow_mappings() {
    let mut labels = HashMap::new();
    labels.insert("env".to_string(), "prod".to_string());
    labels.insert("app".to_string(), "myapp".to_string());

    let config = ConfigWithMap {
        name: "server".to_string(),
        labels,
    };

    let yaml =
        to_yaml_string_with_options(&config, &SerializerOptions::with_flow_mappings()).unwrap();

    println!("Generated YAML:\n{}", yaml);

    // Should have labels in flow style (containing braces)
    assert!(yaml.contains("labels:"));
    assert!(
        yaml.contains("{") && yaml.contains("}"),
        "Expected flow style for labels"
    );
}

#[test]
fn test_flow_all() {
    let mut labels = HashMap::new();
    labels.insert("env".to_string(), "prod".to_string());

    #[derive(Serialize)]
    struct AllFlowConfig {
        ports: Vec<i32>,
        labels: HashMap<String, String>,
    }

    let config = AllFlowConfig {
        ports: vec![80, 443],
        labels,
    };

    let yaml = to_yaml_string_with_options(&config, &SerializerOptions::with_flow_all()).unwrap();

    println!("Generated YAML:\n{}", yaml);

    // Both should be in flow style
    assert!(yaml.contains("["), "Expected flow style for sequences");
    assert!(yaml.contains("{"), "Expected flow style for mappings");
}
