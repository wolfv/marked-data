//! Comprehensive tests for marked_yaml serialization

use marked_yaml::{
    from_yaml, node_to_yaml_string, parse_yaml, to_node, to_yaml_string,
    to_yaml_string_with_options, SerializerOptions,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[test]
fn test_serialize_primitives() {
    // String
    let s = "hello world";
    let yaml = to_yaml_string(&s).unwrap();
    assert!(yaml.contains("hello world"));

    // Integer
    let n = 42;
    let yaml = to_yaml_string(&n).unwrap();
    assert_eq!(yaml.trim(), "42");

    // Float
    let f = 3.14;
    let yaml = to_yaml_string(&f).unwrap();
    assert!(yaml.contains("3.14"));

    // Boolean
    let b = true;
    let yaml = to_yaml_string(&b).unwrap();
    assert_eq!(yaml.trim(), "true");

    // None
    let opt: Option<i32> = None;
    let yaml = to_yaml_string(&opt).unwrap();
    // yaml_rust emits ~ for null
    assert!(yaml.trim() == "null" || yaml.trim() == "~");

    // Some
    let opt = Some(42);
    let yaml = to_yaml_string(&opt).unwrap();
    assert_eq!(yaml.trim(), "42");
}

#[test]
fn test_serialize_vec() {
    let vec = vec![1, 2, 3, 4, 5];
    let yaml = to_yaml_string(&vec).unwrap();

    // Verify it's block style by default
    assert!(yaml.contains("- 1"));
    assert!(yaml.contains("- 2"));
    assert!(yaml.contains("- 3"));
}

#[test]
fn test_serialize_vec_of_strings() {
    let vec = vec!["apple", "banana", "cherry"];
    let yaml = to_yaml_string(&vec).unwrap();

    assert!(yaml.contains("- apple"));
    assert!(yaml.contains("- banana"));
    assert!(yaml.contains("- cherry"));
}

#[test]
fn test_serialize_hashmap() {
    let mut map = HashMap::new();
    map.insert("name", "Alice");
    map.insert("age", "30");
    map.insert("city", "New York");

    let yaml = to_yaml_string(&map).unwrap();

    assert!(yaml.contains("name:"));
    assert!(yaml.contains("Alice"));
    assert!(yaml.contains("age:"));
    assert!(yaml.contains("30"));
    assert!(yaml.contains("city:"));
    assert!(yaml.contains("New York"));
}

#[test]
fn test_serialize_struct() {
    #[derive(Serialize)]
    struct Person {
        name: String,
        age: u32,
        active: bool,
    }

    let person = Person {
        name: "Bob".to_string(),
        age: 25,
        active: true,
    };

    let yaml = to_yaml_string(&person).unwrap();

    assert!(yaml.contains("name:"));
    assert!(yaml.contains("Bob"));
    assert!(yaml.contains("age:"));
    assert!(yaml.contains("25"));
    assert!(yaml.contains("active:"));
    assert!(yaml.contains("true"));
}

#[test]
fn test_serialize_nested_struct() {
    #[derive(Serialize)]
    struct Address {
        street: String,
        city: String,
    }

    #[derive(Serialize)]
    struct Person {
        name: String,
        address: Address,
    }

    let person = Person {
        name: "Charlie".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "Boston".to_string(),
        },
    };

    let yaml = to_yaml_string(&person).unwrap();

    assert!(yaml.contains("name:"));
    assert!(yaml.contains("Charlie"));
    assert!(yaml.contains("address:"));
    assert!(yaml.contains("street:"));
    assert!(yaml.contains("123 Main St"));
    assert!(yaml.contains("city:"));
    assert!(yaml.contains("Boston"));
}

#[test]
fn test_serialize_with_vec() {
    #[derive(Serialize)]
    struct Team {
        name: String,
        members: Vec<String>,
    }

    let team = Team {
        name: "Engineering".to_string(),
        members: vec![
            "Alice".to_string(),
            "Bob".to_string(),
            "Charlie".to_string(),
        ],
    };

    let yaml = to_yaml_string(&team).unwrap();

    assert!(yaml.contains("name:"));
    assert!(yaml.contains("Engineering"));
    assert!(yaml.contains("members:"));
    assert!(yaml.contains("- Alice"));
    assert!(yaml.contains("- Bob"));
    assert!(yaml.contains("- Charlie"));
}

#[test]
fn test_serialize_enum() {
    #[derive(Serialize)]
    enum Status {
        Active,
        Inactive,
        Pending,
    }

    let status = Status::Active;
    let yaml = to_yaml_string(&status).unwrap();
    assert_eq!(yaml.trim(), "Active");

    let status = Status::Pending;
    let yaml = to_yaml_string(&status).unwrap();
    assert_eq!(yaml.trim(), "Pending");
}

#[test]
fn test_serialize_enum_with_data() {
    #[derive(Serialize)]
    enum Message {
        Text(String),
        Number(i32),
    }

    let msg = Message::Text("Hello".to_string());
    let yaml = to_yaml_string(&msg).unwrap();
    assert!(yaml.contains("Text:"));
    assert!(yaml.contains("Hello"));

    let msg = Message::Number(42);
    let yaml = to_yaml_string(&msg).unwrap();
    assert!(yaml.contains("Number:"));
    assert!(yaml.contains("42"));
}

#[test]
fn test_flow_style_sequences() {
    let vec = vec![1, 2, 3];

    let mut options = SerializerOptions::default();
    options.flow_sequences = true;

    let yaml = to_yaml_string_with_options(&vec, &options).unwrap();

    // Should be in flow style: [1, 2, 3]
    assert!(yaml.contains("["));
    assert!(yaml.contains("]"));
    assert!(yaml.contains("1"));
    assert!(yaml.contains("2"));
    assert!(yaml.contains("3"));
}

#[test]
fn test_flow_style_mappings() {
    let mut map = HashMap::new();
    map.insert("a", 1);
    map.insert("b", 2);

    let mut options = SerializerOptions::default();
    options.flow_mappings = true;

    let yaml = to_yaml_string_with_options(&map, &options).unwrap();

    // Should be in flow style: {a: 1, b: 2} or similar
    assert!(yaml.contains("{"));
    assert!(yaml.contains("}"));
}

#[test]
fn test_roundtrip_serialization() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Config {
        version: i32,
        name: String,
        enabled: bool,
    }

    let config = Config {
        version: 1,
        name: "test".to_string(),
        enabled: true,
    };

    // Serialize
    let yaml = to_yaml_string(&config).unwrap();

    // Deserialize
    let parsed: Config = from_yaml(0, &yaml).unwrap();

    // Should match
    assert_eq!(config.version, parsed.version);
    assert_eq!(config.name, parsed.name);
    assert_eq!(config.enabled, parsed.enabled);
}

#[test]
fn test_roundtrip_with_nested_data() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Inner {
        value: i32,
    }

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Outer {
        name: String,
        inner: Inner,
        items: Vec<i32>,
    }

    let data = Outer {
        name: "test".to_string(),
        inner: Inner { value: 42 },
        items: vec![1, 2, 3],
    };

    // Serialize
    let yaml = to_yaml_string(&data).unwrap();

    // Deserialize
    let parsed: Outer = from_yaml(0, &yaml).unwrap();

    // Should match
    assert_eq!(data.name, parsed.name);
    assert_eq!(data.inner.value, parsed.inner.value);
    assert_eq!(data.items, parsed.items);
}

#[test]
fn test_to_node_conversion() {
    let mut map = HashMap::new();
    map.insert("key", "value");

    let node = to_node(&map).unwrap();

    // Should be a mapping
    let mapping = node.as_mapping().expect("Should be a mapping");

    // Should contain the key
    let value = mapping.get_scalar("key").expect("Should have key");
    assert_eq!(value.as_str(), "value");
}

#[test]
fn test_node_to_yaml_string() {
    // Create a node manually
    let node = parse_yaml(0, "key: value\nanother: 123").unwrap();

    // Convert to YAML string
    let options = SerializerOptions::default();
    let yaml = node_to_yaml_string(&node, &options).unwrap();

    // Should contain both keys
    assert!(yaml.contains("key:"));
    assert!(yaml.contains("value"));
    assert!(yaml.contains("another:"));
    assert!(yaml.contains("123"));
}

#[test]
fn test_special_characters_in_strings() {
    let map = HashMap::from([
        ("colon", "value: with colon"),
        ("hash", "value # with hash"),
        ("brackets", "value [with] brackets"),
    ]);

    let yaml = to_yaml_string(&map).unwrap();

    // These should be quoted
    assert!(yaml.contains("\"value: with colon\""));
    assert!(yaml.contains("\"value # with hash\""));
    assert!(yaml.contains("\"value [with] brackets\""));
}

#[test]
fn test_empty_containers() {
    let empty_vec: Vec<i32> = vec![];
    let yaml = to_yaml_string(&empty_vec).unwrap();
    assert!(yaml.contains("[]"));

    let empty_map: HashMap<String, String> = HashMap::new();
    let yaml = to_yaml_string(&empty_map).unwrap();
    assert!(yaml.contains("{}"));
}

#[test]
fn test_null_values() {
    let opt: Option<String> = None;
    let yaml = to_yaml_string(&opt).unwrap();
    assert!(yaml.trim() == "null" || yaml.trim() == "~");
}

#[test]
fn test_serialize_tuple() {
    let tuple = (1, "two", 3.0);
    let yaml = to_yaml_string(&tuple).unwrap();

    // Tuples serialize as sequences
    assert!(yaml.contains("- 1"));
    assert!(yaml.contains("- two"));
    assert!(yaml.contains("- 3"));
}

#[test]
fn test_newlines_in_strings() {
    let text = "line1\nline2\nline3";
    let yaml = to_yaml_string(&text).unwrap();

    // Newlines should be escaped in the output
    assert!(yaml.contains("\\n"));
}

#[test]
fn test_unicode_strings() {
    let text = "Hello 世界 🌍";
    let yaml = to_yaml_string(&text).unwrap();

    // Unicode should be preserved
    assert!(yaml.contains("世界"));
    assert!(yaml.contains("🌍"));
}
