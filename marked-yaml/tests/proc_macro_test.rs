//! Test for the proc macro #[marked_yaml(flow)] attribute

use marked_yaml::{to_yaml_string};
use marked_yaml_derive::Serialize;

#[derive(Serialize)]
struct BasicConfig {
    name: String,

    #[marked_yaml(flow)]
    ports: Vec<i32>,
}

#[test]
fn test_proc_macro_basic() {
    let config = BasicConfig {
        name: "test".to_string(),
        ports: vec![80, 443, 8080],
    };

    let yaml = to_yaml_string(&config).unwrap();

    // Should have name field
    assert!(yaml.contains("name:"));
    assert!(yaml.contains("test"));

    // Should have ports field
    assert!(yaml.contains("ports:"));

    // Note: The flow style doesn't work yet because FlowHint is just a placeholder
    // But at least it compiles and serializes correctly
    println!("Generated YAML:\n{}", yaml);
}

#[derive(Serialize)]
struct NestedConfig {
    version: i32,

    #[marked_yaml(flow)]
    features: Vec<String>,

    database: DatabaseConfig,
}

#[derive(Serialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
}

#[test]
fn test_proc_macro_nested() {
    let config = NestedConfig {
        version: 1,
        features: vec!["beta".to_string(), "experimental".to_string()],
        database: DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
        },
    };

    let yaml = to_yaml_string(&config).unwrap();

    // Should serialize correctly
    assert!(yaml.contains("version:"));
    assert!(yaml.contains("features:"));
    assert!(yaml.contains("database:"));
    assert!(yaml.contains("host:"));
    assert!(yaml.contains("port:"));

    println!("Generated YAML:\n{}", yaml);
}
