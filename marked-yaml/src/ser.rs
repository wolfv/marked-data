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
    types::{MappingHash, MarkedMappingNode, MarkedScalarNode, MarkedSequenceNode},
    Node, Span,
};
use serde::ser::{
    Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
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

/// Serialization options for controlling YAML output format
#[derive(Clone, Debug)]
pub struct SerializerOptions {
    /// Whether to use flow style (inline) for **all** sequences
    ///
    /// When true, sequences like `vec![1, 2, 3]` will be output as `[1, 2, 3]`
    /// instead of:
    /// ```yaml
    /// - 1
    /// - 2
    /// - 3
    /// ```
    pub flow_sequences: bool,

    /// Whether to use flow style (inline) for **all** mappings
    ///
    /// When true, mappings will be output as `{key: value}` instead of:
    /// ```yaml
    /// key: value
    /// ```
    pub flow_mappings: bool,
}

impl SerializerOptions {
    /// Enable flow style for all sequences
    ///
    /// # Example
    ///
    /// ```
    /// use marked_yaml::{to_yaml_string_with_options, SerializerOptions};
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Config {
    ///     ports: Vec<i32>,
    /// }
    ///
    /// let mut options = SerializerOptions::default();
    /// options.flow_sequences = true;
    ///
    /// let config = Config { ports: vec![80, 443, 8080] };
    /// let yaml = to_yaml_string_with_options(&config, &options).unwrap();
    /// assert!(yaml.contains("[80, 443, 8080]"));
    /// ```
    pub fn with_flow_sequences() -> Self {
        Self {
            flow_sequences: true,
            flow_mappings: false,
        }
    }

    /// Enable flow style for all mappings
    pub fn with_flow_mappings() -> Self {
        Self {
            flow_sequences: false,
            flow_mappings: true,
        }
    }

    /// Enable flow style for both sequences and mappings
    pub fn with_flow_all() -> Self {
        Self {
            flow_sequences: true,
            flow_mappings: true,
        }
    }
}

impl Default for SerializerOptions {
    fn default() -> Self {
        Self {
            flow_sequences: false,
            flow_mappings: false,
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
    let node = to_node_with_options(value, options)?;
    node_to_yaml_string(&node, options)
}

/// Convert a `Node` to a YAML string
///
/// This uses our custom emitter which supports flow style control and produces
/// more compact, standard-compliant output.
pub fn node_to_yaml_string(node: &Node, options: &SerializerOptions) -> Result<String, SerError> {
    // Always use our custom emitter for consistent, well-formatted output
    // Benefits:
    // - Supports per-node flow style control
    // - Produces compact list formatting (no blank lines after dashes)
    // - Consistent behavior regardless of options
    // - No dependency on yaml_rust for serialization
    Ok(emit_yaml(node, options))
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
    T: Serialize + ?Sized,
{
    let serializer = NodeSerializer {
        options: options.clone(),
    };
    value.serialize(serializer)
}

// No thread-local storage needed for the serialize_with approach!

/// Implement Serialize for Node so that when a Node is serialized through
/// a NodeSerializer, it returns itself directly, preserving all metadata
/// including style information
impl Serialize for Node {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Node::Scalar(scalar) => {
                // Serialize scalars as their underlying values
                let s = scalar.as_str();
                if let Some(b) = scalar.as_bool() {
                    serializer.serialize_bool(b)
                } else if let Ok(i) = s.parse::<i64>() {
                    serializer.serialize_i64(i)
                } else if let Ok(f) = s.parse::<f64>() {
                    serializer.serialize_f64(f)
                } else {
                    serializer.serialize_str(s)
                }
            }
            Node::Sequence(seq) => {
                // Serialize sequences - preserve the MarkedSequenceNode structure
                // by implementing Serialize on it
                seq.serialize(serializer)
            }
            Node::Mapping(map) => {
                // Serialize mappings - preserve the MarkedMappingNode structure
                map.serialize(serializer)
            }
        }
    }
}

/// Implement Serialize for MarkedSequenceNode to preserve style metadata
/// This is used when a Node with style information is re-serialized
impl Serialize for MarkedSequenceNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Use a wrapper struct to pass the sequence with metadata through
        // The serializer will call serialize_newtype_struct, and we can
        // return the sequence as a Node from there
        serializer.serialize_newtype_struct("__MarkedSequenceNode", &PreservedSeq(self))
    }
}

/// Wrapper to serialize a sequence while preserving its style
/// Uses the style marker approach to communicate style to SeqSerializer
struct PreservedSeq<'a>(&'a MarkedSequenceNode);

impl<'a> Serialize for PreservedSeq<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        // Calculate size including optional style marker
        let size = if self.0.style.is_some() {
            self.0.len() + 1
        } else {
            self.0.len()
        };

        let mut seq = serializer.serialize_seq(Some(size))?;

        // FIRST: serialize a style marker if we have style metadata
        #[cfg(feature = "serde")]
        if let Some(style) = self.0.style {
            seq.serialize_element(&StyleMarker(style))?;
        }

        // Then serialize all the actual elements
        for item in self.0.iter() {
            seq.serialize_element(item)?;
        }

        seq.end()
    }
}

/// Implement Serialize for MarkedMappingNode to preserve style metadata
/// This is used when a Node with style information is re-serialized
impl Serialize for MarkedMappingNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Use a wrapper struct to pass the mapping with metadata through
        serializer.serialize_newtype_struct("__MarkedMappingNode", &PreservedMap(self))
    }
}

/// Wrapper to serialize a mapping while preserving its style
/// Uses the style marker approach to communicate style to MapSerializer
struct PreservedMap<'a>(&'a MarkedMappingNode);

impl<'a> Serialize for PreservedMap<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        // Calculate size including optional style marker
        let size = if self.0.style.is_some() {
            self.0.len() + 1
        } else {
            self.0.len()
        };

        let mut map = serializer.serialize_map(Some(size))?;

        // FIRST: serialize a style marker if we have style metadata
        #[cfg(feature = "serde")]
        if let Some(style) = self.0.style {
            map.serialize_entry("__style_marker", &StyleMarker(style))?;
        }

        // Then serialize all the actual entries
        for (k, v) in self.0.iter() {
            map.serialize_entry(k.as_str(), v)?;
        }

        map.end()
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
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: Serialize,
    {
        // Special handling for preserved nodes - just serialize them directly
        // This allows us to return Nodes with style metadata intact
        if name == "__marked_yaml_preserved_node"
            || name == "__MarkedSequenceNode"
            || name == "__MarkedMappingNode"
        {
            // The value is a Node or contains a Node - serialize it and return directly
            value.serialize(self)
        } else {
            value.serialize(self)
        }
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
        Ok(SeqSerializer {
            nodes: Vec::with_capacity(len.unwrap_or(0)),
            options: self.options,
            #[cfg(feature = "serde")]
            style: None,
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
        Ok(MapSerializer {
            map: MappingHash::new(),
            next_key: None,
            options: self.options,
            #[cfg(feature = "serde")]
            style: None,
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
            #[cfg(feature = "serde")]
            style: None,
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
            #[cfg(feature = "serde")]
            style: None,
        })
    }
}

/// Serializer for sequences
struct SeqSerializer {
    nodes: Vec<Node>,
    options: SerializerOptions,
    #[cfg(feature = "serde")]
    style: Option<crate::types::YamlNodeStyle>,
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

        // Check if this is a style marker (will be a scalar with special value)
        #[cfg(feature = "serde")]
        if self.nodes.is_empty() {
            // This might be a style marker - check if it's a scalar with __Flow or __Block
            if let Node::Scalar(ref scalar) = node {
                let s = scalar.as_str();
                if s == "__Flow" {
                    self.style = Some(crate::types::YamlNodeStyle::Flow);
                    return Ok(()); // Don't add to nodes
                } else if s == "__Block" {
                    self.style = Some(crate::types::YamlNodeStyle::Block);
                    return Ok(()); // Don't add to nodes
                }
            }
        }

        self.nodes.push(node);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        #[cfg(feature = "serde")]
        {
            let mut seq_node = MarkedSequenceNode::new(Span::new_blank(), self.nodes);
            seq_node.style = self.style;
            Ok(Node::Sequence(seq_node))
        }
        #[cfg(not(feature = "serde"))]
        {
            Ok(Node::Sequence(MarkedSequenceNode::new(
                Span::new_blank(),
                self.nodes,
            )))
        }
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
    #[cfg(feature = "serde")]
    style: Option<crate::types::YamlNodeStyle>,
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
        let key = self.next_key.take().ok_or_else(|| {
            SerError::Custom("serialize_value called before serialize_key".to_string())
        })?;

        // Check if this is the style marker key
        #[cfg(feature = "serde")]
        if key.as_str() == "__style_marker" && self.map.is_empty() {
            // This is a style marker - extract the style
            let node = value.serialize(NodeSerializer {
                options: self.options.clone(),
            })?;
            if let Node::Scalar(ref scalar) = node {
                let s = scalar.as_str();
                if s == "__Flow" {
                    self.style = Some(crate::types::YamlNodeStyle::Flow);
                    return Ok(()); // Don't add to map
                } else if s == "__Block" {
                    self.style = Some(crate::types::YamlNodeStyle::Block);
                    return Ok(()); // Don't add to map
                }
            }
        }

        let node = value.serialize(NodeSerializer {
            options: self.options.clone(),
        })?;
        self.map.insert(key, node);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        #[cfg(feature = "serde")]
        {
            let mut map_node = MarkedMappingNode::new(Span::new_blank(), self.map);
            map_node.style = self.style;
            Ok(Node::Mapping(map_node))
        }
        #[cfg(not(feature = "serde"))]
        {
            Ok(Node::Mapping(MarkedMappingNode::new(
                Span::new_blank(),
                self.map,
            )))
        }
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
    emit_node(&mut output, node, 0, options, false, false);
    output
}

/// Recursively emit a node to the output string
fn emit_node(
    output: &mut String,
    node: &Node,
    indent: usize,
    options: &SerializerOptions,
    inline: bool,
    skip_initial_newline: bool,
) {
    match node {
        Node::Scalar(scalar) => {
            emit_scalar(output, scalar);
        }
        Node::Sequence(seq) => {
            // Check if this node has explicit flow style set
            #[cfg(feature = "serde")]
            let node_wants_flow = seq
                .style
                .map(|s| s == crate::types::YamlNodeStyle::Flow)
                .unwrap_or(false);
            #[cfg(not(feature = "serde"))]
            let node_wants_flow = false;

            let use_flow = node_wants_flow || options.flow_sequences || inline;

            if seq.is_empty() {
                output.push_str("[]");
            } else if use_flow {
                // Flow style (inline): [item1, item2, item3]
                output.push('[');
                for (i, item) in seq.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    emit_node(output, item, indent, options, true, false);
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
                        // Scalars: put on the same line as the dash
                        emit_node(output, item, indent + 1, options, false, false);
                    } else {
                        // Complex types (maps/sequences): skip the initial newline
                        // so they start on the same line as the dash
                        emit_node(output, item, indent + 1, options, false, true);
                    }
                }
            }
        }
        Node::Mapping(map) => {
            // Check if this node has explicit flow style set
            #[cfg(feature = "serde")]
            let node_wants_flow = map
                .style
                .map(|s| s == crate::types::YamlNodeStyle::Flow)
                .unwrap_or(false);
            #[cfg(not(feature = "serde"))]
            let node_wants_flow = false;

            let use_flow = node_wants_flow || options.flow_mappings || inline;

            if map.is_empty() {
                output.push_str("{}");
            } else if use_flow {
                // Flow style (inline): {key1: value1, key2: value2}
                output.push('{');
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        output.push_str(", ");
                    }
                    emit_scalar(output, key);
                    output.push_str(": ");
                    emit_node(output, value, indent, options, true, false);
                }
                output.push('}');
            } else {
                // Block style
                for (i, (key, value)) in map.iter().enumerate() {
                    if i == 0 && skip_initial_newline {
                        // First item and we're skipping initial newline (e.g., after a list dash)
                        // The key goes on the same line as the dash
                    } else {
                        output.push('\n');
                        output.push_str(&"  ".repeat(indent));
                    }
                    emit_scalar(output, key);
                    output.push_str(": ");
                    let is_scalar = matches!(value, Node::Scalar(_));
                    if is_scalar {
                        emit_node(output, value, indent + 1, options, false, false);
                    } else {
                        emit_node(output, value, indent + 1, options, false, false);
                    }
                }
            }
        }
    }
}

/// Emit a scalar value, quoting if necessary
fn emit_scalar(output: &mut String, scalar: &crate::types::MarkedScalarNode) {
    let value = scalar.as_str();

    // If may_coerce is true, this scalar can be interpreted as bool/number/null
    // In that case, we should NOT quote boolean-like keywords because they ARE booleans
    let is_typed_value = scalar.may_coerce()
        && matches!(
            value,
            "true" | "false" | "null" | "True" | "False" | "TRUE" | "FALSE" | "NULL" | "~"
        );

    // Check if the value needs quoting
    let needs_quoting = !is_typed_value
        && (value.is_empty()
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
            || value.contains('\n'));

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

/// Serde helper for serializing Vec fields in flow style (inline `[...]`)
///
/// Use with `#[serde(serialize_with = "marked_yaml::as_flow_sequence")]`
///
/// # Example
///
/// ```
/// use serde::Serialize;
/// use marked_yaml::to_yaml_string;
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///     #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
///     ports: Vec<i32>,
/// }
///
/// let config = Config {
///     name: "web-server".to_string(),
///     ports: vec![80, 443, 8080],
/// };
///
/// let yaml = to_yaml_string(&config).unwrap();
/// // Output:
/// // name: web-server
/// // ports: [80, 443, 8080]
/// ```
pub fn as_flow_sequence<T, S>(value: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    T: Serialize,
    S: serde::Serializer,
{
    // Serialize each element to a Node
    let mut nodes = Vec::with_capacity(value.len());
    for item in value {
        let node = to_node(item).map_err(serde::ser::Error::custom)?;
        nodes.push(node);
    }

    // Create a sequence node with flow style
    let flow_node = Node::Sequence(crate::types::MarkedSequenceNode::new_flow(
        crate::Span::new_blank(),
        nodes,
    ));

    // Serialize the node through the parent serializer
    serialize_node(&flow_node, serializer)
}

/// Serde helper for serializing map fields in flow style (inline `{...}`)
///
/// Works with HashMap, BTreeMap, and any other map-like collection.
///
/// Use with `#[serde(serialize_with = "marked_yaml::as_flow_mapping")]`
pub fn as_flow_mapping<'a, K, V, M, S>(value: &'a M, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize + 'a,
    V: Serialize + 'a,
    &'a M: IntoIterator<Item = (&'a K, &'a V)>,
    S: serde::Serializer,
{
    // Serialize to a mapping node
    let mut map = crate::types::MappingHash::new();
    for (k, v) in value {
        let key_node = to_node(k).map_err(serde::ser::Error::custom)?;
        let key_scalar = match key_node {
            Node::Scalar(s) => s,
            _ => {
                return Err(serde::ser::Error::custom(
                    "Map keys must serialize to scalars",
                ))
            }
        };
        let value_node = to_node(v).map_err(serde::ser::Error::custom)?;
        map.insert(key_scalar, value_node);
    }

    // Create a mapping node with flow style
    let flow_node = Node::Mapping(crate::types::MarkedMappingNode::new_flow(
        crate::Span::new_blank(),
        map,
    ));

    serialize_node(&flow_node, serializer)
}

/// Helper to serialize a Node through any serializer
/// For NodeSerializer, this preserves the Node structure including style metadata
fn serialize_node<S>(node: &Node, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Wrap in a DirectNodeWrapper that will return this exact node
    DirectNodeWrapper(node.clone()).serialize(serializer)
}

/// Wrapper that returns a Node directly when serialized
/// This is the KEY to preserving style metadata!
struct DirectNodeWrapper(Node);

impl Serialize for DirectNodeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // The trick: we use serialize_bytes with a special marker,
        // then intercept it in NodeSerializer
        //
        // Actually, better idea: use the fact that the serializer is NodeSerializer
        // and just return the node by serializing its components

        // Clone the node and serialize it component by component
        // This way we preserve the style metadata
        match &self.0 {
            Node::Scalar(s) => s.as_str().serialize(serializer),
            Node::Sequence(seq) => {
                // Return this exact sequence node with its style preserved!
                // We do this by wrapping in DirectSeqWrapper
                DirectSeqWrapper(seq.clone()).serialize(serializer)
            }
            Node::Mapping(map) => DirectMapWrapper(map.clone()).serialize(serializer),
        }
    }
}

/// Wrapper for a sequence that preserves style when serialized
struct DirectSeqWrapper(MarkedSequenceNode);

impl Serialize for DirectSeqWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        // Create a sequence serializer
        let mut seq = serializer.serialize_seq(Some(self.0.len() + 1))?; // +1 for style marker

        // FIRST: serialize a style marker to communicate the style to SeqSerializer!
        #[cfg(feature = "serde")]
        if let Some(style) = self.0.style {
            seq.serialize_element(&StyleMarker(style))?;
        }

        // Then serialize all the actual elements
        for node in self.0.iter() {
            seq.serialize_element(&DirectNodeWrapper(node.clone()))?;
        }

        seq.end()
    }
}

/// A marker type that carries style information
/// When SeqSerializer sees this as the first element, it extracts the style
/// and removes it from the sequence
#[cfg(feature = "serde")]
struct StyleMarker(crate::types::YamlNodeStyle);

#[cfg(feature = "serde")]
impl Serialize for StyleMarker {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Serialize as a unit_variant with a special name
        serializer.serialize_unit_variant(
            "__StyleMarker",
            self.0 as u32,
            match self.0 {
                crate::types::YamlNodeStyle::Block => "__Block",
                crate::types::YamlNodeStyle::Flow => "__Flow",
            },
        )
    }
}

/// Wrapper for a mapping that preserves style when serialized
struct DirectMapWrapper(MarkedMappingNode);

impl Serialize for DirectMapWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        // Create a map serializer (including space for style marker if needed)
        let map_size = if self.0.style.is_some() {
            self.0.len() + 1
        } else {
            self.0.len()
        };
        let mut map = serializer.serialize_map(Some(map_size))?;

        // FIRST: serialize a style marker if we have style info
        #[cfg(feature = "serde")]
        if let Some(style) = self.0.style {
            map.serialize_entry("__style_marker", &StyleMarker(style))?;
        }

        // Then serialize all the actual entries
        for (k, v) in self.0.iter() {
            map.serialize_entry(k.as_str(), &DirectNodeWrapper(v.clone()))?;
        }

        map.end()
    }
}

/// Flow style control for YAML serialization
///
/// There are two main approaches to control flow style (inline formatting):
///
/// ## 1. Per-Field with `serialize_with` (Recommended for Selective Control)
///
/// ```
/// use serde::Serialize;
/// use marked_yaml::to_yaml_string;
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///
///     #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
///     ports: Vec<i32>,
///
///     #[serde(serialize_with = "marked_yaml::as_flow_sequence")]
///     allowed_ips: Vec<String>,
/// }
/// ```
///
/// ## 2. Global Options (For All Fields of a Type)
///
/// Use `SerializerOptions` to control flow style globally:
///
/// ```rust
/// use serde::Serialize;
/// use marked_yaml::{to_yaml_string_with_options, SerializerOptions};
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///     ports: Vec<i32>,
/// }
///
/// let config = Config {
///     name: "web-server".to_string(),
///     ports: vec![80, 443, 8080],
/// };
///
/// // ALL sequences will use flow style
/// let yaml = to_yaml_string_with_options(
///     &config,
///     &SerializerOptions::with_flow_sequences()
/// ).unwrap();
/// ```

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
