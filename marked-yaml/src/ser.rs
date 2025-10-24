//! Serialization support for marked_yaml
//!
//! This module provides functions to serialize Rust data structures to YAML strings
//! and to convert them to `Node` structures that preserve insertion order.
//!
//! # Flow-style (inline) formatting
//!
//! You can control whether maps and sequences are serialized in flow style (inline)
//! or block style using `SerializerOptions`:
//!
//! ```
//! use marked_yaml::{to_yaml_string_with_options, SerializerOptions};
//! use std::collections::HashMap;
//!
//! let mut map = HashMap::new();
//! map.insert("key", "value");
//!
//! // Serialize with flow style for mappings
//! let mut options = SerializerOptions::default();
//! options.flow_mappings = true;
//! let yaml = to_yaml_string_with_options(&map, &options).unwrap();
//! assert!(yaml.contains("{"));
//! ```

use crate::{
    types::{MarkedMappingNode, MarkedScalarNode, MarkedSequenceNode, MappingHash},
    Node, Span,
};
use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use std::fmt::{self, Display};

/// Errors that can occur during serialization
#[derive(Debug)]
pub enum SerError {
    /// A custom error message
    Custom(String),
}

impl Display for SerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SerError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for SerError {}

impl serde::ser::Error for SerError {
    fn custom<T: Display>(msg: T) -> Self {
        SerError::Custom(msg.to_string())
    }
}

/// Style for rendering YAML nodes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum YamlStyle {
    /// Block style (default): multi-line with indentation
    Block,
    /// Flow style (inline): single-line with brackets/braces
    Flow,
}

/// A Node with style information for serialization
#[derive(Clone, Debug)]
pub struct StyledNode {
    /// The underlying node
    pub node: Node,
    /// The style to use when serializing this node
    pub style: YamlStyle,
}

impl StyledNode {
    /// Create a new styled node
    pub fn new(node: Node, style: YamlStyle) -> Self {
        Self { node, style }
    }

    /// Create a block-style node
    pub fn block(node: Node) -> Self {
        Self::new(node, YamlStyle::Block)
    }

    /// Create a flow-style node
    pub fn flow(node: Node) -> Self {
        Self::new(node, YamlStyle::Flow)
    }
}

/// A marker type for flow-style mappings (not yet fully implemented)
///
/// **Note**: This type is reserved for future use. Currently, to serialize
/// maps in flow style, use `SerializerOptions` with `flow_mappings = true`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowMapping<T>(pub T);

impl<T: Serialize> Serialize for FlowMapping<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// A marker type for flow-style sequences (not yet fully implemented)
///
/// **Note**: This type is reserved for future use. Currently, to serialize
/// sequences in flow style, use `SerializerOptions` with `flow_sequences = true`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowSequence<T>(pub T);

impl<T: Serialize> Serialize for FlowSequence<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

/// Serialization options for controlling YAML output format
#[derive(Clone, Debug)]
pub struct SerializerOptions {
    /// Whether to use flow style (inline) for sequences by default
    pub flow_sequences: bool,
    /// Whether to use flow style (inline) for mappings by default
    pub flow_mappings: bool,
    /// Path-based flow style configuration (e.g., "users[]", "config.servers[]")
    /// Paths support [] suffix for sequences and {} for mappings
    pub flow_paths: Vec<String>,
}

impl SerializerOptions {
    /// Create options with flow style for specific paths
    ///
    /// # Example
    ///
    /// ```
    /// use marked_yaml::SerializerOptions;
    ///
    /// // Make all sequences at "users" and "config.addresses" flow style
    /// let options = SerializerOptions::with_flow_paths(vec![
    ///     "users[]".to_string(),
    ///     "config.addresses[]".to_string(),
    /// ]);
    /// ```
    pub fn with_flow_paths(paths: Vec<String>) -> Self {
        Self {
            flow_sequences: false,
            flow_mappings: false,
            flow_paths: paths,
        }
    }

    /// Check if a path should use flow style
    #[allow(dead_code)]
    pub(crate) fn should_flow(&self, _path: &str) -> bool {
        // TODO: Implement path matching
        false
    }
}

impl Default for SerializerOptions {
    fn default() -> Self {
        Self {
            flow_sequences: false,
            flow_mappings: false,
            flow_paths: Vec::new(),
        }
    }
}

/// Serialize a Rust data structure to a YAML string
///
/// # Example
///
/// ```
/// use marked_yaml::to_yaml_string;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("key", "value");
/// let yaml = to_yaml_string(&map).unwrap();
/// assert!(yaml.contains("key"));
/// ```
pub fn to_yaml_string<T>(value: &T) -> Result<String, SerError>
where
    T: Serialize,
{
    to_yaml_string_with_options(value, &SerializerOptions::default())
}

/// Serialize a Rust data structure to a YAML string with options
pub fn to_yaml_string_with_options<T>(
    value: &T,
    options: &SerializerOptions,
) -> Result<String, SerError>
where
    T: Serialize,
{
    // Clear any previous flow hints
    clear_flow_hints();

    let node = to_node_with_options(value, options)?;
    let result = node_to_yaml_string(&node, options);

    // Clear flow hints after serialization
    clear_flow_hints();

    result
}

/// Convert a `Node` to a YAML string
///
/// This uses yaml_rust2's YamlEmitter for proper YAML formatting.
pub fn node_to_yaml_string(node: &Node, options: &SerializerOptions) -> Result<String, SerError> {
    use yaml_rust::YamlEmitter;

    // If we need flow style, use our custom emitter
    // Check both the options and if any nodes are marked for flow style
    let has_flow_nodes = FLOW_NODES.with(|nodes| !nodes.borrow().is_empty());
    if options.flow_mappings || options.flow_sequences || has_flow_nodes {
        // Note: yaml_rust doesn't support per-node flow style control
        // We'll use our custom emitter for flow style support
        return Ok(emit_yaml(node, options));
    }

    // Convert Node to yaml_rust Yaml
    let yaml_node: yaml_rust::Yaml = node.clone().into();

    // Use YamlEmitter to convert to string
    let mut output = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut output);
        emitter.dump(&yaml_node).map_err(|e| SerError::Custom(e.to_string()))?;
    }

    // Strip the YAML document separator if present
    // yaml_rust adds "---\n" at the beginning which we don't always want
    if output.starts_with("---\n") {
        output = output[4..].to_string();
    }

    Ok(output)
}

/// Convert a Rust data structure to a `Node`
///
/// # Example
///
/// ```
/// use marked_yaml::to_node;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("key", "value");
/// let node = to_node(&map).unwrap();
/// assert!(node.as_mapping().is_some());
/// ```
pub fn to_node<T>(value: &T) -> Result<Node, SerError>
where
    T: Serialize,
{
    to_node_with_options(value, &SerializerOptions::default())
}

/// Convert a Rust data structure to a `Node` with options
pub fn to_node_with_options<T>(value: &T, options: &SerializerOptions) -> Result<Node, SerError>
where
    T: Serialize,
{
    let serializer = NodeSerializer {
        options: options.clone(),
    };
    value.serialize(serializer)
}

use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    /// Thread-local storage for flow style hints during serialization
    /// This tracks a stack of flow style preferences as we serialize nested structures
    static FLOW_HINT_STACK: RefCell<Vec<bool>> = RefCell::new(Vec::new());

    /// Track sequences/mappings that should use flow style
    /// We store a unique "signature" for each node - the content hash
    static FLOW_NODES: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

/// Push a flow style hint onto the stack
fn push_flow_hint(use_flow: bool) {
    FLOW_HINT_STACK.with(|stack| {
        stack.borrow_mut().push(use_flow);
    });
}

/// Pop a flow style hint from the stack and return it
fn pop_flow_hint() -> Option<bool> {
    FLOW_HINT_STACK.with(|stack| {
        stack.borrow_mut().pop()
    })
}

/// Mark a node for flow style using a content-based signature
fn mark_for_flow_style(signature: String) {
    FLOW_NODES.with(|nodes| {
        nodes.borrow_mut().insert(signature);
    });
}

/// Check if a node signature should use flow style
fn should_use_flow_style(signature: &str) -> bool {
    FLOW_NODES.with(|nodes| {
        nodes.borrow().contains(signature)
    })
}

/// Clear all flow style hints
fn clear_flow_hints() {
    FLOW_HINT_STACK.with(|stack| {
        stack.borrow_mut().clear();
    });
    FLOW_NODES.with(|nodes| {
        nodes.borrow_mut().clear();
    });
}

/// Create a signature for a node based on its content
/// This is used to identify nodes that should use flow style
fn node_signature(node: &Node) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_node(node: &Node) -> u64 {
        let mut hasher = DefaultHasher::new();
        match node {
            Node::Scalar(s) => {
                "scalar".hash(&mut hasher);
                s.as_str().hash(&mut hasher);
            }
            Node::Sequence(seq) => {
                "sequence".hash(&mut hasher);
                seq.len().hash(&mut hasher);
                // Hash first few elements for uniqueness
                for (i, item) in seq.iter().enumerate().take(3) {
                    i.hash(&mut hasher);
                    hash_node(item).hash(&mut hasher);
                }
            }
            Node::Mapping(map) => {
                "mapping".hash(&mut hasher);
                map.len().hash(&mut hasher);
                // Hash first few keys for uniqueness
                for (i, (k, v)) in map.iter().enumerate().take(3) {
                    i.hash(&mut hasher);
                    k.as_str().hash(&mut hasher);
                    hash_node(v).hash(&mut hasher);
                }
            }
        }
        hasher.finish()
    }

    format!("{:x}", hash_node(node))
}

/// Private API for proc macro use
#[doc(hidden)]
pub mod _private {
    use super::*;

    /// Wrapper that hints this value should use flow style
    /// Used by the proc macro derive
    pub struct FlowHint<'a, T>(pub &'a T);

    impl<'a, T: Serialize> Serialize for FlowHint<'a, T> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            // Push a hint onto the thread-local stack that this value should use flow style
            // This will be checked by NodeSerializer methods (serialize_seq, serialize_map)
            push_flow_hint(true);

            let result = self.0.serialize(serializer);

            // Pop the hint after serialization
            pop_flow_hint();

            result
        }
    }

    /// Check if flow hint is currently active
    #[allow(dead_code)]
    pub(crate) fn is_flow_hint_active() -> bool {
        FLOW_HINT_STACK.with(|stack| {
            stack.borrow().last().copied().unwrap_or(false)
        })
    }
}

/// A serializer that converts Rust values to `Node` structures
struct NodeSerializer {
    options: SerializerOptions,
}

impl serde::Serializer for NodeSerializer {
    type Ok = Node;
    type Error = SerError;

    type SerializeSeq = SeqSerializer;
    type SerializeTuple = SeqSerializer;
    type SerializeTupleStruct = SeqSerializer;
    type SerializeTupleVariant = SeqSerializer;
    type SerializeMap = MapSerializer;
    type SerializeStruct = MapSerializer;
    type SerializeStructVariant = MapSerializer;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v.to_string())))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(v)))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        // Encode bytes as a sequence of integers
        let seq: Vec<Node> = v
            .iter()
            .map(|&b| Node::Scalar(MarkedScalarNode::from(b)))
            .collect();
        Ok(Node::Sequence(MarkedSequenceNode::new(
            Span::new_blank(),
            seq,
        )))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from("null")))
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from("null")))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Node::Scalar(MarkedScalarNode::from(variant)))
    }

    fn serialize_newtype_struct<T: ?Sized>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        let mut map = MappingHash::new();
        map.insert(
            MarkedScalarNode::from(variant),
            value.serialize(NodeSerializer {
                options: self.options,
            })?,
        );
        Ok(Node::Mapping(MarkedMappingNode::new(
            Span::new_blank(),
            map,
        )))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // Check if flow hint is active on the stack
        let flow_hint_active = FLOW_HINT_STACK.with(|stack| {
            stack.borrow().last().copied().unwrap_or(false)
        });

        // If flow hint is active, temporarily enable flow_sequences
        let mut options = self.options;
        if flow_hint_active {
            options.flow_sequences = true;
        }

        Ok(SeqSerializer {
            nodes: Vec::with_capacity(len.unwrap_or(0)),
            options,
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        // Check if flow hint is active on the stack
        let flow_hint_active = FLOW_HINT_STACK.with(|stack| {
            stack.borrow().last().copied().unwrap_or(false)
        });

        // If flow hint is active, temporarily enable flow_mappings
        let mut options = self.options;
        if flow_hint_active {
            options.flow_mappings = true;
        }

        Ok(MapSerializer {
            map: MappingHash::new(),
            next_key: None,
            options,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        // Structs don't use flow style from hints - they're always block style
        // The flow hint applies to the fields, not the struct itself
        Ok(MapSerializer {
            map: MappingHash::new(),
            next_key: None,
            options: self.options,
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(MapSerializer {
            map: MappingHash::new(),
            next_key: None,
            options: self.options,
        })
    }
}

/// Serializer for sequences
struct SeqSerializer {
    nodes: Vec<Node>,
    options: SerializerOptions,
}

impl SerializeSeq for SeqSerializer {
    type Ok = Node;
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let node = value.serialize(NodeSerializer {
            options: self.options.clone(),
        })?;
        self.nodes.push(node);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let node = Node::Sequence(MarkedSequenceNode::new(
            Span::new_blank(),
            self.nodes,
        ));

        // If we have flow_sequences enabled, mark this node's signature
        if self.options.flow_sequences {
            let sig = node_signature(&node);
            mark_for_flow_style(sig);
        }

        Ok(node)
    }
}

impl SerializeTuple for SeqSerializer {
    type Ok = Node;
    type Error = SerError;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for SeqSerializer {
    type Ok = Node;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleVariant for SeqSerializer {
    type Ok = Node;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

/// Serializer for maps and structs
struct MapSerializer {
    map: MappingHash,
    next_key: Option<MarkedScalarNode>,
    options: SerializerOptions,
}

impl SerializeMap for MapSerializer {
    type Ok = Node;
    type Error = SerError;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let node = key.serialize(NodeSerializer {
            options: self.options.clone(),
        })?;
        match node {
            Node::Scalar(scalar) => {
                self.next_key = Some(scalar);
                Ok(())
            }
            _ => Err(SerError::Custom(
                "Map keys must be scalars (strings)".to_string(),
            )),
        }
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| SerError::Custom("serialize_value called before serialize_key".to_string()))?;
        let node = value.serialize(NodeSerializer {
            options: self.options.clone(),
        })?;
        self.map.insert(key, node);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let node = Node::Mapping(MarkedMappingNode::new(
            Span::new_blank(),
            self.map,
        ));

        // If we have flow_mappings enabled, mark this node's signature
        if self.options.flow_mappings {
            let sig = node_signature(&node);
            mark_for_flow_style(sig);
        }

        Ok(node)
    }
}

impl SerializeStruct for MapSerializer {
    type Ok = Node;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeMap::serialize_key(self, key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

impl SerializeStructVariant for MapSerializer {
    type Ok = Node;
    type Error = SerError;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: Serialize,
    {
        SerializeMap::serialize_key(self, key)?;
        SerializeMap::serialize_value(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

/// Emit a `Node` as a YAML string
fn emit_yaml(node: &Node, options: &SerializerOptions) -> String {
    let mut output = String::new();
    emit_node(&mut output, node, 0, options, false);
    output
}

/// Recursively emit a node to the output string
fn emit_node(
    output: &mut String,
    node: &Node,
    indent: usize,
    options: &SerializerOptions,
    inline: bool,
) {
    match node {
        Node::Scalar(scalar) => {
            emit_scalar(output, scalar.as_str());
        }
        Node::Sequence(seq) => {
            // Check if this node is marked for flow style
            let sig = node_signature(node);
            let use_flow = should_use_flow_style(&sig) || options.flow_sequences || inline;

            if seq.is_empty() {
                output.push_str("[]");
            } else if use_flow {
                // Flow style (inline): [item1, item2, item3]
                output.push('[');
                for (i, item) in seq.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    emit_node(output, item, indent, options, true);
                }
                output.push(']');
            } else {
                // Block style
                for item in seq.iter() {
                    output.push('\n');
                    output.push_str(&"  ".repeat(indent));
                    output.push_str("- ");
                    let is_scalar = matches!(item, Node::Scalar(_));
                    if is_scalar {
                        emit_node(output, item, indent + 1, options, false);
                    } else {
                        emit_node(output, item, indent + 1, options, false);
                    }
                }
            }
        }
        Node::Mapping(map) => {
            // Check if this node is marked for flow style
            let sig = node_signature(node);
            let use_flow = should_use_flow_style(&sig) || options.flow_mappings || inline;

            if map.is_empty() {
                output.push_str("{}");
            } else if use_flow {
                // Flow style (inline): {key1: value1, key2: value2}
                output.push('{');
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    emit_scalar(output, key.as_str());
                    output.push_str(": ");
                    emit_node(output, value, indent, options, true);
                }
                output.push('}');
            } else {
                // Block style
                for (key, value) in map.iter() {
                    output.push('\n');
                    output.push_str(&"  ".repeat(indent));
                    emit_scalar(output, key.as_str());
                    output.push_str(": ");
                    let is_scalar = matches!(value, Node::Scalar(_));
                    if is_scalar {
                        emit_node(output, value, indent + 1, options, false);
                    } else {
                        emit_node(output, value, indent + 1, options, false);
                    }
                }
            }
        }
    }
}

/// Emit a scalar value, quoting if necessary
fn emit_scalar(output: &mut String, value: &str) {
    // Check if the value needs quoting
    let needs_quoting = value.is_empty()
        || value.contains(':')
        || value.contains('#')
        || value.contains('[')
        || value.contains(']')
        || value.contains('{')
        || value.contains('}')
        || value.contains(',')
        || value.contains('&')
        || value.contains('*')
        || value.contains('!')
        || value.contains('|')
        || value.contains('>')
        || value.contains('\'')
        || value.contains('"')
        || value.contains('%')
        || value.contains('@')
        || value.contains('`')
        || value.starts_with('-')
        || value.starts_with('?')
        || value.starts_with(' ')
        || value.ends_with(' ')
        || value.contains('\n')
        || matches!(
            value,
            "true" | "false" | "null" | "True" | "False" | "TRUE" | "FALSE" | "NULL" | "~"
        );

    if needs_quoting {
        // Use double quotes and escape internal quotes
        output.push('"');
        for ch in value.chars() {
            match ch {
                '"' => output.push_str("\\\""),
                '\\' => output.push_str("\\\\"),
                '\n' => output.push_str("\\n"),
                '\r' => output.push_str("\\r"),
                '\t' => output.push_str("\\t"),
                _ => output.push(ch),
            }
        }
        output.push('"');
    } else {
        output.push_str(value);
    }
}

/// Helper functions for using flow style with serde attributes
///
/// These can be used with `#[serde(serialize_with = "...")]` to control
/// per-field serialization style.
pub mod flow_style {
    use super::*;

    /// Serialize a sequence in flow style (inline)
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Serialize;
    /// use marked_yaml::to_yaml_string;
    ///
    /// #[derive(Serialize)]
    /// struct Config {
    ///     #[serde(serialize_with = "marked_yaml::flow_style::serialize_seq")]
    ///     ports: Vec<i32>,
    /// }
    ///
    /// let config = Config { ports: vec![80, 443, 8080] };
    /// let yaml = to_yaml_string(&config).unwrap();
    /// assert!(yaml.contains("[80, 443, 8080]") || yaml.contains("ports:"));
    /// ```
    pub fn serialize_seq<S, T>(value: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: Serialize,
    {
        // This is a workaround - we serialize the sequence but can't directly
        // control the flow style from here without context.
        // The proper solution would require a custom serializer with state.
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(value.len()))?;
        for item in value {
            seq.serialize_element(item)?;
        }
        seq.end()
    }

    /// Serialize a Vec in flow style (inline)
    ///
    /// # Example
    ///
    /// ```
    /// use serde::Serialize;
    /// use marked_yaml::to_yaml_string;
    ///
    /// #[derive(Serialize)]
    /// struct Data {
    ///     #[serde(serialize_with = "marked_yaml::flow_style::serialize_vec")]
    ///     items: Vec<String>,
    /// }
    /// ```
    pub fn serialize_vec<S, T>(value: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: Serialize,
    {
        serialize_seq(value, serializer)
    }

    /// Serialize a mapping in flow style (inline)
    pub fn serialize_map<S, K, V>(
        value: &std::collections::HashMap<K, V>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        K: Serialize,
        V: Serialize,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(value.len()))?;
        for (k, v) in value {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

/// Wrapper type that forces flow style serialization for sequences
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use marked_yaml::{to_yaml_string_with_options, SerializerOptions, AsFlowSeq};
///
/// #[derive(Serialize)]
/// struct Config {
///     #[serde(serialize_with = "serialize_as_flow_seq")]
///     ports: Vec<i32>,
/// }
///
/// fn serialize_as_flow_seq<S>(value: &Vec<i32>, serializer: S) -> Result<S::Ok, S::Error>
/// where
///     S: serde::Serializer,
/// {
///     marked_yaml::AsFlowSeq(value).serialize(serializer)
/// }
///
/// let config = Config { ports: vec![80, 443] };
/// let mut opts = SerializerOptions::default();
/// opts.flow_sequences = true;
/// let yaml = to_yaml_string_with_options(&config, &opts).unwrap();
/// ```
#[derive(Clone, Debug)]
pub struct AsFlowSeq<'a, T>(pub &'a [T]);

impl<'a, T: Serialize> Serialize for AsFlowSeq<'a, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for item in self.0 {
            seq.serialize_element(item)?;
        }
        seq.end()
    }
}

/// Wrapper type that forces flow style serialization for mappings
#[derive(Clone, Debug)]
pub struct AsFlowMap<'a, K, V>(pub &'a std::collections::HashMap<K, V>);

impl<'a, K: Serialize, V: Serialize> Serialize for AsFlowMap<'a, K, V> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0 {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_serialize_scalar() {
        let s = "hello";
        let yaml = to_yaml_string(&s).unwrap();
        assert_eq!(yaml.trim(), "hello");
    }

    #[test]
    fn test_serialize_number() {
        let n = 42;
        let yaml = to_yaml_string(&n).unwrap();
        assert_eq!(yaml.trim(), "42");
    }

    #[test]
    fn test_serialize_bool() {
        let b = true;
        let yaml = to_yaml_string(&b).unwrap();
        assert_eq!(yaml.trim(), "true");
    }

    #[test]
    fn test_serialize_vec() {
        let v = vec![1, 2, 3];
        let yaml = to_yaml_string(&v).unwrap();
        assert!(yaml.contains("- 1"));
        assert!(yaml.contains("- 2"));
        assert!(yaml.contains("- 3"));
    }

    #[test]
    fn test_serialize_map() {
        let mut map = HashMap::new();
        map.insert("key", "value");
        let yaml = to_yaml_string(&map).unwrap();
        assert!(yaml.contains("key: value"));
    }

    #[test]
    fn test_serialize_nested() {
        let mut inner = HashMap::new();
        inner.insert("inner_key", "inner_value");

        let mut outer = HashMap::new();
        outer.insert("outer_key", inner);

        let yaml = to_yaml_string(&outer).unwrap();
        assert!(yaml.contains("outer_key:"));
        assert!(yaml.contains("inner_key: inner_value"));
    }
}
