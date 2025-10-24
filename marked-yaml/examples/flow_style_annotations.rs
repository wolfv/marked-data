//! Example demonstrating flow style annotations for specific fields

use marked_yaml::{to_yaml_string, to_yaml_string_with_options, SerializerOptions};
use serde::Serialize;

/// Using global SerializerOptions for flow style
#[derive(Serialize)]
struct ConfigWithOptions {
    name: String,
    version: i32,
    // These will be affected by global flow_sequences option
    ports: Vec<i32>,
    servers: Vec<String>,
}

/// Helper function to serialize a specific field in flow style
/// This is a workaround - we use global options for now
fn example_with_global_options() {
    let config = ConfigWithOptions {
        name: "MyApp".to_string(),
        version: 1,
        ports: vec![80, 443, 8080],
        servers: vec!["server1".to_string(), "server2".to_string()],
    };

    println!("=== Block Style (Default) ===\n");
    let yaml = to_yaml_string(&config).unwrap();
    println!("{}\n", yaml);

    println!("=== All Sequences in Flow Style ===\n");
    let mut options = SerializerOptions::default();
    options.flow_sequences = true;
    let yaml = to_yaml_string_with_options(&config, &options).unwrap();
    println!("{}\n", yaml);

    println!("=== All Mappings in Flow Style ===\n");
    let mut options = SerializerOptions::default();
    options.flow_mappings = true;
    let yaml = to_yaml_string_with_options(&config, &options).unwrap();
    println!("{}\n", yaml);
}

/// Nested structure example
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
    addresses: Vec<Address>,
}

fn example_with_nested_structures() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
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

    println!("=== Nested Structures - Block Style ===\n");
    let yaml = to_yaml_string(&person).unwrap();
    println!("{}\n", yaml);

    println!("=== Nested Structures - Flow Style for All Sequences ===\n");
    let mut options = SerializerOptions::default();
    options.flow_sequences = true;
    let yaml = to_yaml_string_with_options(&person, &options).unwrap();
    println!("{}\n", yaml);

    println!("=== Nested Structures - Flow Style for All Mappings ===\n");
    let mut options = SerializerOptions::default();
    options.flow_mappings = true;
    let yaml = to_yaml_string_with_options(&person, &options).unwrap();
    println!("{}\n", yaml);
}

/// Example using path-based configuration (future feature)
fn example_path_based_flow_style() {
    println!("=== Path-Based Flow Style (Future Feature) ===\n");
    println!("Example usage (not yet implemented):\n");
    println!("let options = SerializerOptions::with_flow_paths(vec![");
    println!("    \"addresses[]\".to_string(),  // Flow style for addresses array");
    println!("    \"config.servers[]\".to_string(),  // Flow for nested arrays");
    println!("    \"metadata{{}}\".to_string(),  // Flow for specific map");
    println!("]);\n");
    println!("This would allow fine-grained control over which specific");
    println!("fields should use flow style vs block style.\n");
}

fn main() {
    example_with_global_options();
    println!("\n{}\n", "=".repeat(60));
    example_with_nested_structures();
    println!("\n{}\n", "=".repeat(60));
    example_path_based_flow_style();

    println!("\n{}\n", "=".repeat(60));
    println!("=== Current Best Practice ===\n");
    println!("For now, the recommended approach is to use SerializerOptions");
    println!("to control flow style globally:");
    println!("  - options.flow_sequences = true  // All arrays inline");
    println!("  - options.flow_mappings = true   // All objects inline");
    println!("\nFuture enhancements will add:");
    println!("  1. Path-based configuration for specific fields");
    println!("  2. Proc macro attributes like #[marked_yaml(flow)]");
    println!("  3. Runtime hints via wrapper types\n");
}
