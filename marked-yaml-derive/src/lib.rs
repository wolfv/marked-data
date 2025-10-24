//! Proc macro for marked-yaml serialization attributes
//!
//! This crate provides derive macros for controlling YAML serialization format
//! using attributes like `#[marked_yaml(flow)]`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Derive macro for Serialize with marked_yaml attributes
///
/// # Attributes
///
/// - `#[marked_yaml(flow)]` - Serialize this field in flow style (inline)
/// - `#[marked_yaml(block)]` - Explicitly use block style (default)
///
/// # Example
///
/// ```ignore
/// use marked_yaml_derive::Serialize;
/// use serde::Serialize as _;
///
/// #[derive(Serialize)]
/// struct Config {
///     name: String,
///
///     #[marked_yaml(flow)]
///     ports: Vec<i32>,
///
///     #[marked_yaml(flow)]
///     servers: Vec<String>,
///
///     // Regular fields use block style
///     nested: NestedConfig,
/// }
/// ```
#[proc_macro_derive(Serialize, attributes(marked_yaml))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_serialize(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn expand_serialize(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Tuple structs are not yet supported",
                ));
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Unit structs are not supported",
                ));
            }
        },
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Enums are not yet supported. Use #[derive(serde::Serialize)] instead.",
            ));
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(input, "Unions are not supported"));
        }
    };

    // Analyze fields for flow style attributes
    let mut field_serializations = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();

        // Check for #[marked_yaml(flow)] attribute
        let has_flow = field.attrs.iter().any(|attr| {
            if !attr.path().is_ident("marked_yaml") {
                return false;
            }

            // Try to parse as an identifier: #[marked_yaml(flow)]
            if let Ok(nested) = attr.parse_args::<syn::Ident>() {
                return nested == "flow";
            }

            false
        });

        if has_flow {
            // For flow-style fields, we need special handling
            // For now, we'll use a helper function that serializes with flow hints
            field_serializations.push(quote! {
                map.serialize_entry(
                    #field_name_str,
                    &marked_yaml::_private::FlowHint(&self.#field_name)
                )?;
            });
        } else {
            // Regular serialization
            field_serializations.push(quote! {
                map.serialize_entry(#field_name_str, &self.#field_name)?;
            });
        }
    }

    let field_count = fields.len();

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::serde::Serialize for #name #ty_generics #where_clause {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                use ::serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(::std::option::Option::Some(#field_count))?;
                #(#field_serializations)*
                map.end()
            }
        }
    };

    Ok(expanded)
}
