use anyhow::{Context, Result};
use openapiv3::{
    IntegerFormat, NumberFormat, OpenAPI, ReferenceOr, Schema, SchemaKind, StringFormat, Type,
    VariantOrUnknownOrEmpty,
};

use crate::spec::{LoadedSpec, SpecVersion};
use crate::tree::{NodeInfo, NodeKind, TreeNode};

pub fn parse(content: &str) -> Result<LoadedSpec> {
    let api: OpenAPI = serde_yaml::from_str(content).context("failed to parse OpenAPI 3.x spec")?;

    let version = if api.openapi.starts_with("3.1") {
        SpecVersion::V31
    } else {
        SpecVersion::V30
    };

    let title = api.info.title.clone();
    let spec_version_str = api.openapi.clone();

    // Extract top-level schema names (sorted) — kept for tests
    let schema_names: Vec<String> = api
        .components
        .as_ref()
        .map(|c| {
            let mut names: Vec<String> = c.schemas.keys().cloned().collect();
            names.sort();
            names
        })
        .unwrap_or_default();

    let schema_nodes = build_tree(&api);

    Ok(LoadedSpec {
        title,
        openapi_version: spec_version_str,
        version,
        schema_names,
        schema_nodes,
    })
}

// ── tree construction ─────────────────────────────────────────────────────────

fn build_tree(api: &OpenAPI) -> Vec<TreeNode> {
    api.components
        .as_ref()
        .map(|c| {
            let mut schemas: Vec<(&String, &ReferenceOr<Schema>)> =
                c.schemas.iter().collect();
            schemas.sort_by_key(|(k, _)| k.as_str());
            schemas
                .into_iter()
                .map(|(name, schema_ref)| schema_ref_to_node(name.clone(), schema_ref, false))
                .collect()
        })
        .unwrap_or_default()
}

/// Convert a `ReferenceOr<Schema>` (used in components.schemas) into a TreeNode.
fn schema_ref_to_node(name: String, schema_ref: &ReferenceOr<Schema>, required: bool) -> TreeNode {
    match schema_ref {
        ReferenceOr::Reference { reference } => ref_node(name, reference, required),
        ReferenceOr::Item(schema) => schema_to_node(name, schema, required),
    }
}

/// Convert a `ReferenceOr<Box<Schema>>` (used in properties / array items) into a TreeNode.
fn boxed_schema_ref_to_node(
    name: String,
    schema_ref: &ReferenceOr<Box<Schema>>,
    required: bool,
) -> TreeNode {
    match schema_ref {
        ReferenceOr::Reference { reference } => ref_node(name, reference, required),
        ReferenceOr::Item(schema) => schema_to_node(name, schema, required),
    }
}

fn ref_node(name: String, reference: &str, required: bool) -> TreeNode {
    let target = extract_ref_name(reference);
    TreeNode {
        name,
        info: NodeInfo {
            kind: NodeKind::Ref(target),
            format: None,
            description: None,
            required,
            constraints: Vec::new(),
            enum_values: Vec::new(),
            example: None,
            default_val: None,
        },
        children: Vec::new(),
        expanded: false,
    }
}

fn schema_to_node(name: String, schema: &Schema, required: bool) -> TreeNode {
    let data = &schema.schema_data;
    let description = data.description.clone();
    let example = data
        .example
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());
    let default_val = data
        .default
        .as_ref()
        .map(|v| serde_json::to_string(v).unwrap_or_default());

    match &schema.schema_kind {
        SchemaKind::Type(Type::Object(obj)) => {
            let children: Vec<TreeNode> = obj
                .properties
                .iter()
                .map(|(prop_name, prop_ref)| {
                    let is_req = obj.required.contains(prop_name);
                    boxed_schema_ref_to_node(prop_name.clone(), prop_ref, is_req)
                })
                .collect();

            let mut constraints = Vec::new();
            if let Some(n) = obj.min_properties {
                constraints.push(format!("minProperties: {n}"));
            }
            if let Some(n) = obj.max_properties {
                constraints.push(format!("maxProperties: {n}"));
            }

            TreeNode {
                name,
                info: NodeInfo {
                    kind: NodeKind::Object,
                    format: None,
                    description,
                    required,
                    constraints,
                    enum_values: Vec::new(),
                    example,
                    default_val,
                },
                children,
                expanded: false,
            }
        }

        SchemaKind::Type(Type::Array(arr)) => {
            let mut children = Vec::new();
            if let Some(items_ref) = &arr.items {
                children.push(boxed_schema_ref_to_node("items".to_string(), items_ref, false));
            }

            let mut constraints = Vec::new();
            if let Some(n) = arr.min_items {
                constraints.push(format!("minItems: {n}"));
            }
            if let Some(n) = arr.max_items {
                constraints.push(format!("maxItems: {n}"));
            }
            if arr.unique_items {
                constraints.push("uniqueItems: true".to_string());
            }

            TreeNode {
                name,
                info: NodeInfo {
                    kind: NodeKind::Array,
                    format: None,
                    description,
                    required,
                    constraints,
                    enum_values: Vec::new(),
                    example,
                    default_val,
                },
                children,
                expanded: false,
            }
        }

        SchemaKind::Type(Type::String(s)) => {
            let format = match &s.format {
                VariantOrUnknownOrEmpty::Item(f) => Some(string_format_label(f).to_string()),
                VariantOrUnknownOrEmpty::Unknown(u) => Some(u.clone()),
                VariantOrUnknownOrEmpty::Empty => None,
            };

            let mut constraints = Vec::new();
            if let Some(n) = s.min_length {
                constraints.push(format!("minLength: {n}"));
            }
            if let Some(n) = s.max_length {
                constraints.push(format!("maxLength: {n}"));
            }
            if let Some(pat) = &s.pattern {
                constraints.push(format!("pattern: {pat}"));
            }

            let enum_values: Vec<String> = s
                .enumeration
                .iter()
                .filter_map(|v| v.as_deref().map(|s| format!("\"{s}\"")))
                .collect();

            TreeNode {
                name,
                info: NodeInfo {
                    kind: NodeKind::Str,
                    format,
                    description,
                    required,
                    constraints,
                    enum_values,
                    example,
                    default_val,
                },
                children: Vec::new(),
                expanded: false,
            }
        }

        SchemaKind::Type(Type::Integer(i)) => {
            let format = match &i.format {
                VariantOrUnknownOrEmpty::Item(f) => Some(integer_format_label(f).to_string()),
                VariantOrUnknownOrEmpty::Unknown(u) => Some(u.clone()),
                VariantOrUnknownOrEmpty::Empty => None,
            };

            let mut constraints = Vec::new();
            if let Some(min) = i.minimum {
                if i.exclusive_minimum {
                    constraints.push(format!("min: >{min}"));
                } else {
                    constraints.push(format!("min: {min}"));
                }
            }
            if let Some(max) = i.maximum {
                if i.exclusive_maximum {
                    constraints.push(format!("max: <{max}"));
                } else {
                    constraints.push(format!("max: {max}"));
                }
            }
            if let Some(mul) = i.multiple_of {
                constraints.push(format!("multipleOf: {mul}"));
            }

            let enum_values: Vec<String> = i
                .enumeration
                .iter()
                .filter_map(|v| v.map(|n| n.to_string()))
                .collect();

            TreeNode {
                name,
                info: NodeInfo {
                    kind: NodeKind::Integer,
                    format,
                    description,
                    required,
                    constraints,
                    enum_values,
                    example,
                    default_val,
                },
                children: Vec::new(),
                expanded: false,
            }
        }

        SchemaKind::Type(Type::Number(n)) => {
            let format = match &n.format {
                VariantOrUnknownOrEmpty::Item(f) => Some(number_format_label(f).to_string()),
                VariantOrUnknownOrEmpty::Unknown(u) => Some(u.clone()),
                VariantOrUnknownOrEmpty::Empty => None,
            };

            let mut constraints = Vec::new();
            if let Some(min) = n.minimum {
                if n.exclusive_minimum {
                    constraints.push(format!("min: >{min}"));
                } else {
                    constraints.push(format!("min: {min}"));
                }
            }
            if let Some(max) = n.maximum {
                if n.exclusive_maximum {
                    constraints.push(format!("max: <{max}"));
                } else {
                    constraints.push(format!("max: {max}"));
                }
            }
            if let Some(mul) = n.multiple_of {
                constraints.push(format!("multipleOf: {mul}"));
            }

            let enum_values: Vec<String> = n
                .enumeration
                .iter()
                .filter_map(|v| v.map(|f| f.to_string()))
                .collect();

            TreeNode {
                name,
                info: NodeInfo {
                    kind: NodeKind::Number,
                    format,
                    description,
                    required,
                    constraints,
                    enum_values,
                    example,
                    default_val,
                },
                children: Vec::new(),
                expanded: false,
            }
        }

        SchemaKind::Type(Type::Boolean(_)) => TreeNode {
            name,
            info: NodeInfo {
                kind: NodeKind::Boolean,
                format: None,
                description,
                required,
                constraints: Vec::new(),
                enum_values: Vec::new(),
                example,
                default_val,
            },
            children: Vec::new(),
            expanded: false,
        },

        SchemaKind::AllOf { all_of } => {
            let children: Vec<TreeNode> = all_of
                .iter()
                .enumerate()
                .map(|(i, s)| schema_ref_to_node(format!("[{i}]"), s, false))
                .collect();
            TreeNode {
                name,
                info: NodeInfo {
                    kind: NodeKind::AllOf,
                    format: None,
                    description,
                    required,
                    constraints: Vec::new(),
                    enum_values: Vec::new(),
                    example,
                    default_val,
                },
                children,
                expanded: false,
            }
        }

        SchemaKind::OneOf { one_of } => {
            let children: Vec<TreeNode> = one_of
                .iter()
                .enumerate()
                .map(|(i, s)| schema_ref_to_node(format!("[{i}]"), s, false))
                .collect();
            TreeNode {
                name,
                info: NodeInfo {
                    kind: NodeKind::OneOf,
                    format: None,
                    description,
                    required,
                    constraints: Vec::new(),
                    enum_values: Vec::new(),
                    example,
                    default_val,
                },
                children,
                expanded: false,
            }
        }

        SchemaKind::AnyOf { any_of } => {
            let children: Vec<TreeNode> = any_of
                .iter()
                .enumerate()
                .map(|(i, s)| schema_ref_to_node(format!("[{i}]"), s, false))
                .collect();
            TreeNode {
                name,
                info: NodeInfo {
                    kind: NodeKind::AnyOf,
                    format: None,
                    description,
                    required,
                    constraints: Vec::new(),
                    enum_values: Vec::new(),
                    example,
                    default_val,
                },
                children,
                expanded: false,
            }
        }

        // Not / Any → Unknown leaf
        _ => TreeNode {
            name,
            info: NodeInfo {
                kind: NodeKind::Unknown,
                format: None,
                description,
                required,
                constraints: Vec::new(),
                enum_values: Vec::new(),
                example,
                default_val,
            },
            children: Vec::new(),
            expanded: false,
        },
    }
}

// ── format helpers ────────────────────────────────────────────────────────────

fn extract_ref_name(reference: &str) -> String {
    reference
        .rsplit('/')
        .next()
        .unwrap_or(reference)
        .to_string()
}

fn string_format_label(f: &StringFormat) -> &'static str {
    match f {
        StringFormat::Date => "date",
        StringFormat::DateTime => "date-time",
        StringFormat::Password => "password",
        StringFormat::Byte => "byte",
        StringFormat::Binary => "binary",
    }
}

fn integer_format_label(f: &IntegerFormat) -> &'static str {
    match f {
        IntegerFormat::Int32 => "int32",
        IntegerFormat::Int64 => "int64",
    }
}

fn number_format_label(f: &NumberFormat) -> &'static str {
    match f {
        NumberFormat::Float => "float",
        NumberFormat::Double => "double",
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const PETSTORE: &str = include_str!("../../fixtures/petstore.yaml");

    #[test]
    fn parses_petstore_schema_names() {
        let spec = parse(PETSTORE).expect("should parse petstore fixture");
        assert_eq!(spec.title, "Petstore");
        // Should find Error, NewPet, Pet, Pets (sorted alphabetically)
        assert_eq!(spec.schema_names, vec!["Error", "NewPet", "Pet", "Pets"]);
    }

    #[test]
    fn detects_openapi_30() {
        let spec = parse(PETSTORE).expect("should parse petstore fixture");
        assert_eq!(spec.version, SpecVersion::V30);
    }

    #[test]
    fn builds_pet_tree() {
        let spec = parse(PETSTORE).expect("should parse petstore fixture");
        // Find Pet node
        let pet = spec
            .schema_nodes
            .iter()
            .find(|n| n.name == "Pet")
            .expect("Pet schema should exist");
        assert!(matches!(pet.info.kind, NodeKind::Object));
        assert_eq!(pet.children.len(), 3); // id, name, tag
        // id should be required integer/int64
        let id = pet.children.iter().find(|n| n.name == "id").unwrap();
        assert!(matches!(id.info.kind, NodeKind::Integer));
        assert!(id.info.required);
        assert_eq!(id.info.format.as_deref(), Some("int64"));
    }

    #[test]
    fn builds_pets_array_with_ref() {
        let spec = parse(PETSTORE).expect("should parse petstore fixture");
        let pets = spec
            .schema_nodes
            .iter()
            .find(|n| n.name == "Pets")
            .expect("Pets schema should exist");
        assert!(matches!(pets.info.kind, NodeKind::Array));
        assert_eq!(pets.children.len(), 1);
        let items = &pets.children[0];
        assert!(matches!(&items.info.kind, NodeKind::Ref(t) if t == "Pet"));
    }
}
