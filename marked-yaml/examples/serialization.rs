//! Example demonstrating serialization features in marked_yaml

use marked_yaml::{from_yaml, to_yaml_string, to_yaml_string_with_options, SerializerOptions};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    version: i32,
    name: String,
    settings: Settings,
    users: Vec<User>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Settings {
    enabled: bool,
    max_connections: u32,
    timeout_seconds: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    username: String,
    role: Role,
    active: bool,
}

#[derive(Serialize, Deserialize, Debug)]
enum Role {
    Admin,
    User,
    Guest,
}

fn main() {
    // Create a configuration structure
    let config = Config {
        version: 1,
        name: "My Application".to_string(),
        settings: Settings {
            enabled: true,
            max_connections: 100,
            timeout_seconds: 30.5,
        },
        users: vec![
            User {
                username: "alice".to_string(),
                role: Role::Admin,
                active: true,
            },
            User {
                username: "bob".to_string(),
                role: Role::User,
                active: true,
            },
            User {
                username: "charlie".to_string(),
                role: Role::Guest,
                active: false,
            },
        ],
    };

    println!("=== Basic Serialization (Block Style) ===\n");
    let yaml = to_yaml_string(&config).unwrap();
    println!("{}", yaml);

    println!("\n=== Serialization with Flow Style for Maps ===\n");
    let mut options = SerializerOptions::default();
    options.flow_mappings = true;
    let yaml_flow = to_yaml_string_with_options(&config, &options).unwrap();
    println!("{}", yaml_flow);

    println!("\n=== Serialization with Flow Style for Sequences ===\n");
    let mut options = SerializerOptions::default();
    options.flow_sequences = true;
    let yaml_flow_seq = to_yaml_string_with_options(&config, &options).unwrap();
    println!("{}", yaml_flow_seq);

    println!("\n=== Round-trip Test ===\n");
    // Serialize to YAML
    let yaml = to_yaml_string(&config).unwrap();

    // Deserialize back
    let parsed: Config = from_yaml(0, &yaml).unwrap();

    println!("Original version: {}", config.version);
    println!("Parsed version: {}", parsed.version);
    println!("Original name: {}", config.name);
    println!("Parsed name: {}", parsed.name);
    println!("Original user count: {}", config.users.len());
    println!("Parsed user count: {}", parsed.users.len());

    println!("\n=== Simple Data Structures ===\n");

    // Serialize a vector
    let numbers = vec![1, 2, 3, 4, 5];
    println!("Vector: {}", to_yaml_string(&numbers).unwrap());

    // Serialize a HashMap
    let mut map = HashMap::new();
    map.insert("key1", "value1");
    map.insert("key2", "value2");
    println!("HashMap: {}", to_yaml_string(&map).unwrap());

    // Serialize primitives
    println!("String: {}", to_yaml_string(&"Hello, World!").unwrap());
    println!("Integer: {}", to_yaml_string(&42).unwrap());
    println!("Boolean: {}", to_yaml_string(&true).unwrap());
    println!("Float: {}", to_yaml_string(&3.14159).unwrap());
}
