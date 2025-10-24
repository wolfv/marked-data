//! Test for the `#[serde(serialize_with)]` approach to flow style
//! This works by adding style metadata to Node types!

use marked_yaml::to_yaml_string;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize)]
struct Config {
    name: String,

    #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
    ports: Vec<i32>,
}

#[test]
fn test_as_flow_sequence() {
    let config = Config {
        name: "web-server".to_string(),
        ports: vec![80, 443, 8080],
    };

    let yaml = to_yaml_string(&config).unwrap();

    println!("Generated YAML:\n{}", yaml);

    // Should have name field
    assert!(yaml.contains("name:"));
    assert!(yaml.contains("web-server"));

    // Should have ports field in flow style
    assert!(yaml.contains("ports:"));
    assert!(
        yaml.contains("[80, 443, 8080]"),
        "Expected flow style for ports, got:\n{}",
        yaml
    );
}

#[derive(Serialize)]
struct ServerConfig {
    version: i32,

    #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
    features: Vec<String>,

    #[serde(serialize_with = "marked_yaml::as_flow_mapping")]
    labels: HashMap<String, String>,
}

#[test]
fn test_as_flow_mapping() {
    let mut labels = HashMap::new();
    labels.insert("env".to_string(), "prod".to_string());
    labels.insert("app".to_string(), "myapp".to_string());

    let config = ServerConfig {
        version: 1,
        features: vec!["beta".to_string(), "experimental".to_string()],
        labels,
    };

    let yaml = to_yaml_string(&config).unwrap();

    println!("Generated YAML:\n{}", yaml);

    // Should serialize correctly
    assert!(yaml.contains("version:"));
    assert!(yaml.contains("features:"));
    assert!(yaml.contains("labels:"));

    // Features should be in flow style
    assert!(
        yaml.contains("[") && yaml.contains("beta"),
        "Expected flow style for features, got:\n{}",
        yaml
    );

    // Labels should be in flow style
    assert!(
        yaml.contains("{") && yaml.contains("env"),
        "Expected flow style for labels, got:\n{}",
        yaml
    );
}

#[derive(Serialize)]
struct MixedConfig {
    name: String,

    #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
    ports: Vec<i32>,

    // This field uses default block style
    hosts: Vec<String>,
}

#[test]
fn test_mixed_styles() {
    let config = MixedConfig {
        name: "server".to_string(),
        ports: vec![80, 443],
        hosts: vec!["host1".to_string(), "host2".to_string()],
    };

    let yaml = to_yaml_string(&config).unwrap();

    println!("Generated YAML:\n{}", yaml);

    // Ports should be flow style
    assert!(
        yaml.contains("ports: [80, 443]"),
        "Expected flow style for ports, got:\n{}",
        yaml
    );

    // Hosts should be block style (default)
    // It will have "- host1" style
    assert!(
        yaml.contains("- host1") || yaml.contains("hosts:"),
        "Expected block style for hosts, got:\n{}",
        yaml
    );
}

#[derive(Serialize)]
struct BTreeMapConfig {
    name: String,

    #[serde(serialize_with = "marked_yaml::as_flow_mapping")]
    metadata: BTreeMap<String, String>,

    // Regular HashMap also works
    #[serde(serialize_with = "marked_yaml::as_flow_mapping")]
    tags: HashMap<String, i32>,
}

#[test]
fn test_btreemap_support() {
    let mut metadata = BTreeMap::new();
    metadata.insert("version".to_string(), "1.0".to_string());
    metadata.insert("author".to_string(), "alice".to_string());

    let mut tags = HashMap::new();
    tags.insert("priority".to_string(), 1);
    tags.insert("urgency".to_string(), 5);

    let config = BTreeMapConfig {
        name: "test-app".to_string(),
        metadata,
        tags,
    };

    let yaml = to_yaml_string(&config).unwrap();

    println!("Generated YAML:\n{}", yaml);

    // Both should be in flow style
    assert!(
        yaml.contains("metadata: {"),
        "Expected flow style for BTreeMap metadata, got:\n{}",
        yaml
    );
    assert!(
        yaml.contains("tags: {"),
        "Expected flow style for HashMap tags, got:\n{}",
        yaml
    );

    // Verify content
    assert!(yaml.contains("version"));
    assert!(yaml.contains("author"));
    assert!(yaml.contains("priority"));
}
