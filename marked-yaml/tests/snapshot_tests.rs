//! Snapshot tests for YAML serialization output
//!
//! These tests use `insta` to ensure the YAML output format remains consistent
//! and produces well-formatted, standard-compliant YAML.

use insta::assert_snapshot;
use marked_yaml::{
    as_flow_mapping, as_flow_sequence, to_yaml_string, to_yaml_string_with_options,
    SerializerOptions,
};
use serde::Serialize;
use std::collections::BTreeMap;

// ============================================================================
// Basic Types
// ============================================================================

#[test]
fn test_scalar_types() {
    #[derive(Serialize)]
    struct Scalars {
        string: String,
        number: i32,
        float: f64,
        boolean: bool,
        optional_some: Option<i32>,
        optional_none: Option<i32>,
    }

    let data = Scalars {
        string: "hello world".to_string(),
        number: 42,
        float: 3.14,
        boolean: true,
        optional_some: Some(100),
        optional_none: None,
    };

    assert_snapshot!(to_yaml_string(&data).unwrap(), @r###"

    string: hello world
    number: 42
    float: 3.14
    boolean: true
    optional_some: 100
    optional_none: null
    "###);
}

#[test]
fn test_special_string_quoting() {
    #[derive(Serialize)]
    struct SpecialStrings {
        with_colon: String,
        with_hash: String,
        with_brackets: String,
        with_quotes: String,
        url: String,
    }

    let data = SpecialStrings {
        with_colon: "key: value".to_string(),
        with_hash: "comment # here".to_string(),
        with_brackets: "[bracketed]".to_string(),
        with_quotes: r#"has "quotes" inside"#.to_string(),
        url: "https://example.com".to_string(),
    };

    assert_snapshot!(to_yaml_string(&data).unwrap(), @r###"

    with_colon: "key: value"
    with_hash: "comment # here"
    with_brackets: "[bracketed]"
    with_quotes: "has \"quotes\" inside"
    url: "https://example.com"
    "###);
}

// ============================================================================
// Collections - Block Style
// ============================================================================

#[test]
fn test_simple_list_block_style() {
    let list = vec![1, 2, 3, 4, 5];

    assert_snapshot!(to_yaml_string(&list).unwrap(), @r###"

    - 1
    - 2
    - 3
    - 4
    - 5
    "###);
}

#[test]
fn test_list_of_strings_block_style() {
    let list = vec!["apple", "banana", "cherry"];

    assert_snapshot!(to_yaml_string(&list).unwrap(), @r###"

    - apple
    - banana
    - cherry
    "###);
}

#[test]
fn test_map_block_style() {
    let mut map = BTreeMap::new();
    map.insert("name", "Alice");
    map.insert("age", "30");
    map.insert("city", "Boston");

    let yaml = to_yaml_string(&map).unwrap();

    assert_snapshot!(yaml, @r"
    age: 30
    city: Boston
    name: Alice
    ");
}

#[test]
fn test_list_of_objects_compact_format() {
    #[derive(Serialize)]
    struct Person {
        name: String,
        age: i32,
    }

    let people = vec![
        Person {
            name: "Alice".to_string(),
            age: 30,
        },
        Person {
            name: "Bob".to_string(),
            age: 25,
        },
    ];

    assert_snapshot!(to_yaml_string(&people).unwrap(), @r###"

    - name: Alice
      age: 30
    - name: Bob
      age: 25
    "###);
}

// ============================================================================
// Collections - Flow Style (Global Options)
// ============================================================================

#[test]
fn test_flow_sequences_global_option() {
    let list = vec![1, 2, 3, 4, 5];

    let mut options = SerializerOptions::default();
    options.flow_sequences = true;

    assert_snapshot!(to_yaml_string_with_options(&list, &options).unwrap(), @"[1, 2, 3, 4, 5]");
}

#[test]
fn test_flow_mappings_global_option() {
    #[derive(Serialize)]
    struct Simple {
        x: i32,
        y: i32,
    }

    let data = Simple { x: 10, y: 20 };

    let mut options = SerializerOptions::default();
    options.flow_mappings = true;

    assert_snapshot!(to_yaml_string_with_options(&data, &options).unwrap(), @"{x: 10, y: 20}");
}

// ============================================================================
// Per-Field Flow Style Control
// ============================================================================

#[test]
fn test_per_field_flow_sequence() {
    #[derive(Serialize)]
    struct Config {
        name: String,
        #[serde(serialize_with = "as_flow_sequence")]
        ports: Vec<i32>,
        servers: Vec<String>,
    }

    let config = Config {
        name: "web-server".to_string(),
        ports: vec![80, 443, 8080],
        servers: vec!["server1".to_string(), "server2".to_string()],
    };

    assert_snapshot!(to_yaml_string(&config).unwrap(), @r"
    name: web-server
    ports: [80, 443, 8080]
    servers: 
      - server1
      - server2
    ");
}

#[test]
fn test_per_field_flow_mapping() {
    #[derive(Serialize)]
    struct App {
        name: String,
        #[serde(serialize_with = "as_flow_mapping")]
        labels: BTreeMap<String, String>,
        settings: BTreeMap<String, String>,
    }

    let mut labels = BTreeMap::new();
    labels.insert("env".to_string(), "prod".to_string());
    labels.insert("app".to_string(), "myapp".to_string());

    let mut settings = BTreeMap::new();
    settings.insert("timeout".to_string(), "30s".to_string());
    settings.insert("retries".to_string(), "3".to_string());

    let app = App {
        name: "WebApp".to_string(),
        labels,
        settings,
    };

    let yaml = to_yaml_string(&app).unwrap();

    // Flow mapping should be on one line with braces
    assert!(yaml.contains("labels: {"));

    // Regular mapping should be block style
    assert!(yaml.contains("settings: \n"));
}

#[test]
fn test_mixed_flow_and_block_styles() {
    #[derive(Serialize)]
    struct Deployment {
        api_version: String,
        kind: String,
        #[serde(serialize_with = "as_flow_mapping")]
        metadata: BTreeMap<String, String>,
        spec: DeploymentSpec,
    }

    #[derive(Serialize)]
    struct DeploymentSpec {
        replicas: i32,
        #[serde(serialize_with = "as_flow_sequence")]
        ports: Vec<i32>,
        env: BTreeMap<String, String>,
    }

    let mut metadata = BTreeMap::new();
    metadata.insert("name".to_string(), "my-app".to_string());
    metadata.insert("namespace".to_string(), "default".to_string());

    let mut env = BTreeMap::new();
    env.insert("LOG_LEVEL".to_string(), "info".to_string());
    env.insert("DATABASE_URL".to_string(), "postgres://db".to_string());

    let deployment = Deployment {
        api_version: "apps/v1".to_string(),
        kind: "Deployment".to_string(),
        metadata,
        spec: DeploymentSpec {
            replicas: 3,
            ports: vec![8080, 8443],
            env,
        },
    };

    let yaml = to_yaml_string(&deployment).unwrap();

    // Verify flow styles are applied
    assert!(yaml.contains("metadata: {"));
    assert!(yaml.contains("ports: [8080, 8443]"));

    // Verify block style for env
    assert!(yaml.contains("env: \n"));
}

// ============================================================================
// Nested Structures
// ============================================================================

#[test]
fn test_deeply_nested_structure() {
    #[derive(Serialize)]
    struct Root {
        level1: Level1,
    }

    #[derive(Serialize)]
    struct Level1 {
        name: String,
        level2: Level2,
    }

    #[derive(Serialize)]
    struct Level2 {
        items: Vec<String>,
        level3: Level3,
    }

    #[derive(Serialize)]
    struct Level3 {
        value: i32,
    }

    let data = Root {
        level1: Level1 {
            name: "first".to_string(),
            level2: Level2 {
                items: vec!["a".to_string(), "b".to_string()],
                level3: Level3 { value: 42 },
            },
        },
    };

    assert_snapshot!(to_yaml_string(&data).unwrap(), @r"
    level1: 
      name: first
      level2: 
        items: 
          - a
          - b
        level3: 
          value: 42
    ");
}

#[test]
fn test_list_of_maps_compact_formatting() {
    #[derive(Serialize)]
    struct Address {
        street: String,
        city: String,
        zip: String,
    }

    let addresses = vec![
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
    ];

    // This tests the compact list formatting (no blank line after dash)
    assert_snapshot!(to_yaml_string(&addresses).unwrap(), @r###"
    - street: 123 Main St
      city: Boston
      zip: 02101
    - street: 456 Oak Ave
      city: Cambridge
      zip: 02139
    "###);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_collections() {
    #[derive(Serialize)]
    struct Empty {
        empty_vec: Vec<i32>,
        empty_map: BTreeMap<String, String>,
    }

    let data = Empty {
        empty_vec: vec![],
        empty_map: BTreeMap::new(),
    };

    assert_snapshot!(to_yaml_string(&data).unwrap(), @r###"
    empty_vec: []
    empty_map: {}
    "###);
}

#[test]
fn test_single_item_collections() {
    #[derive(Serialize)]
    struct Single {
        one_item: Vec<i32>,
    }

    let data = Single { one_item: vec![42] };

    assert_snapshot!(to_yaml_string(&data).unwrap(), @r"
    one_item: 
      - 42
    ");
}

#[test]
fn test_flow_style_with_nested_structures() {
    #[derive(Serialize)]
    struct Outer {
        #[serde(serialize_with = "as_flow_sequence")]
        coords: Vec<Coord>,
    }

    #[derive(Serialize)]
    struct Coord {
        x: i32,
        y: i32,
    }

    let data = Outer {
        coords: vec![Coord { x: 1, y: 2 }, Coord { x: 3, y: 4 }],
    };

    let yaml = to_yaml_string(&data).unwrap();

    // Flow sequence should contain the coordinate objects
    assert!(yaml.contains("coords: ["));
}

// ============================================================================
// Real-World Examples
// ============================================================================

#[test]
fn test_kubernetes_pod_like_structure() {
    #[derive(Serialize)]
    struct Pod {
        api_version: String,
        kind: String,
        metadata: Metadata,
        spec: PodSpec,
    }

    #[derive(Serialize)]
    struct Metadata {
        name: String,
        namespace: String,
    }

    #[derive(Serialize)]
    struct PodSpec {
        containers: Vec<Container>,
    }

    #[derive(Serialize)]
    struct Container {
        name: String,
        image: String,
        #[serde(serialize_with = "as_flow_sequence")]
        ports: Vec<i32>,
    }

    let pod = Pod {
        api_version: "v1".to_string(),
        kind: "Pod".to_string(),
        metadata: Metadata {
            name: "my-pod".to_string(),
            namespace: "default".to_string(),
        },
        spec: PodSpec {
            containers: vec![Container {
                name: "nginx".to_string(),
                image: "nginx:latest".to_string(),
                ports: vec![80, 443],
            }],
        },
    };

    let yaml = to_yaml_string(&pod).unwrap();

    // Verify structure
    assert!(yaml.contains("api_version: v1"));
    assert!(yaml.contains("kind: Pod"));
    assert!(yaml.contains("containers:"));
    assert!(yaml.contains("ports: [80, 443]"));
}

#[test]
fn test_docker_compose_like_structure() {
    #[derive(Serialize)]
    struct Service {
        image: String,
        #[serde(serialize_with = "as_flow_sequence")]
        ports: Vec<String>,
        environment: BTreeMap<String, String>,
    }

    let mut env = BTreeMap::new();
    env.insert("DATABASE_URL".to_string(), "postgres://db:5432".to_string());
    env.insert("REDIS_URL".to_string(), "redis://cache:6379".to_string());

    let service = Service {
        image: "myapp:latest".to_string(),
        ports: vec!["8080:8080".to_string(), "8443:8443".to_string()],
        environment: env,
    };

    let yaml = to_yaml_string(&service).unwrap();
    assert_snapshot!(yaml, @r#"
    image: "myapp:latest"
    ports: ["8080:8080", "8443:8443"]
    environment: 
      DATABASE_URL: "postgres://db:5432"
      REDIS_URL: "redis://cache:6379"
    "#);
    // Ports should be in flow style
    assert!(yaml.contains("ports: ["));

    // Environment should be in block style
    assert!(yaml.contains("environment: \n"));
}
