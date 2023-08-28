use std::{collections::HashMap, sync::atomic::AtomicU32};

use lazy_static::lazy_static;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::deserializer::{
    AdditionalProperties, Format, PrimitiveType, Schema, SchemaDef, SchemaType,
};

/// A type for a field in a struct
#[derive(Debug, Clone)]
pub enum FieldType {
    /// A field referencing another type, e.g. `MyObjectType`
    /// These field expect the named Types to exist elsewhere in the same scope of the generator.
    /// This variant is also used for classical Enum types, the Enum itself is generated as an Entity.
    Named(String),
    /// A field that is an array of another type or any type
    /// In Python: `list[Any]`, Rust: `Vec<serde_json::Value>` for generic version
    /// or `Vec<f64>` | `Vec<CustomDefinedType>` for specifically typed variants
    Array(Option<Box<FieldType>>),
    /// A Map type with `String` keys and a possible type for the values
    /// If there is no type specified for the value it is assumed to be generic JSON data
    /// In Python: `dict[str, Any]`, Rust: `HashMap<String, serde_json::Value>` for generic version
    /// or `HashMap<String, f64>` | HashMap<String, CustomDefinedType> for specifically typed versions
    Object(Option<Box<FieldType>>),
    /// A Tuple type with a ordered list of types that the values in the Tuple have to be
    /// For example, if we had a `Named("MyType")` and a `Simple(Primitive::Long)`,
    /// the resulting type would be this in rust: `(Box<MyType>, i64)` or in python: `tuple[MyType, int]`
    Tuple(Vec<FieldType>),
    /// A simple type, representing a primitive type of the language that is being used for
    /// generation
    Simple(Primitive),
    /// A constant value for a language primitive type, e.g.
    /// `Const(Primitive::String, "Hello World")` would translate into a field with type:
    /// `MustBe!("Hello World")` in rust or Literal["Hello World"] in python
    Const(Primitive, String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Primitive {
    Int,
    Long,
    Float,
    Double,
    String,
    Bool,
}

/// A type for a field in a struct/class
#[derive(Debug, Clone)]
pub struct Field {
    pub optional: bool,
    pub field_type: FieldType,
}

/// The definition for a Struct/Class like type
#[derive(Debug, Clone)]
pub struct StructDef {
    pub properties: HashMap<String, Field>,
    pub additional_properties: Option<FieldType>,
}

/// Definition for an Enumeration
#[derive(Debug, Clone)]
pub struct EnumDef {
    pub values: Vec<String>,
}

/// A definition for the types that need to be generated
/// `AllOf` and `OneOf` are combinators that need a language-specific solution in the generation step
/// as they can be solved via inheritance/composition or tagged enums (Rust only)
#[derive(Debug, Clone)]
pub enum EntityDef {
    /// A simple definition for a Class-like entity
    Struct(StructDef),
    /// A Collection of Variants and an Optional discriminant
    /// e.g. in Rust the `discriminant` would represent the value inside of
    /// `#[serde(tag="<discriminant>")]`, if not provided `#[serde(untagged)]` is used
    /// Specific values for discriminants that need to be placed in `#[serde(rename="<value>")]`
    /// will be scanned in `Const` fields in the Entity types of the variants
    OneOf {
        discriminant: Option<String>,
        variants: Vec<String>,
    },
    /// AllOf is the inheritance operator, all structs that are combined are referenced by name and
    /// expected to exist.
    AllOf(Vec<String>),
    /// A definition for an Enumeration in a classical sense, a collection of possible values of a
    /// single type
    Enum(EnumDef),
}

/// An entity is any kind of type that needs to be generated in the result code
/// It always has a unique name and a definition
#[derive(Debug, Clone)]
pub struct Entity {
    pub name: String,
    pub def: EntityDef,
}

lazy_static! {
    static ref ANONYMOUS_STRUCT_COUNT: AtomicU32 = AtomicU32::new(1);
    static ref ANONYMOUS_ENUM_COUNT: AtomicU32 = AtomicU32::new(1);
}

fn generate_struct_name() -> String {
    format!(
        "AnonymousEntity{}",
        ANONYMOUS_STRUCT_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

/// Parses a 2nd level and below Schema element into a FieldType and a list of Entities that might be correlated to the
/// field (e.g. anonymous structs that are nested below a field, which will need to be generated or
/// the object type of the field itself that is inlined)
/// It recursively uses `parse_entity` to generate entities for non-primitive types
fn parse_schema(schema: Schema) -> (FieldType, Vec<Entity>) {
    match schema {
        Schema::Ref(schema_ref) => {
            // TODO: handle ref '#' to self for self-referential types
            let name = schema_ref.get_schema_name().to_string();
            (FieldType::Named(name), vec![])
        }
        Schema::Def(schema_def) => match schema_def {
            SchemaDef::Object { ref title, .. }
            | SchemaDef::AllOf { ref title, .. }
            | SchemaDef::OneOf { ref title, .. }
            | SchemaDef::AnyOf { ref title, .. } => {
                let inner_schema_name = title.clone().unwrap_or_else(generate_struct_name);
                (
                    FieldType::Named(inner_schema_name.clone()),
                    parse_entity(schema_def, inner_schema_name),
                )
            }
            SchemaDef::String { type_def, .. } => match type_def {
                PrimitiveType::Const { const_value } => (
                    FieldType::Const(Primitive::String, const_value.clone()),
                    vec![],
                ),
                PrimitiveType::Enum { enum_values } => {
                    let def = EntityDef::Enum(EnumDef {
                        values: enum_values,
                    });
                    let name = format!(
                        "AnonymousEnum{}",
                        ANONYMOUS_ENUM_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                    );
                    let field_type = FieldType::Named(name.clone());
                    let enum_entity = Entity { name, def };
                    (field_type, vec![enum_entity])
                }
                PrimitiveType::Basic { format } => (FieldType::Simple(Primitive::String), vec![]),
            },
            SchemaDef::Integer { type_def, .. } => match type_def {
                PrimitiveType::Const { const_value } => todo!(),
                PrimitiveType::Enum { enum_values } => todo!(),
                PrimitiveType::Basic { format } => (FieldType::Simple(Primitive::Int), vec![]),
            },
            SchemaDef::Array { items, .. } => match items {
                Some(schema) => {
                    let (field_type, entities) = parse_schema(*schema);
                    (FieldType::Array(Some(Box::new(field_type))), entities)
                }
                None => (FieldType::Array(None), vec![]),
            },
            SchemaDef::Tuple { prefix_items, .. } => todo!(),
        },
    }
}

fn parse_combinator_schemas(schemas: Vec<Schema>) -> (Vec<String>, Vec<Entity>) {
    let mut entities = vec![];
    let mut combinator_entities = vec![];
    for schema in schemas {
        match schema {
            Schema::Ref(schema_ref) => {
                let name = schema_ref.get_schema_name().to_string();
                combinator_entities.push(name);
            }
            Schema::Def(schema_def) => {
                let name = match &schema_def {
                    SchemaDef::Object { ref title, .. }
                    | SchemaDef::AllOf { ref title, .. }
                    | SchemaDef::OneOf { ref title, .. }
                    | SchemaDef::AnyOf { ref title, .. } => {
                        title.clone().unwrap_or_else(generate_struct_name)
                    }
                    _ => panic!(
                        "Combinator not supposed to have this type of schema inside: {:?}",
                        schema_def
                    ),
                };

                let mut parsed_entities = parse_entity(schema_def, name.clone());
                entities.append(&mut parsed_entities);
                combinator_entities.push(name);
            }
        }
    }
    (combinator_entities, entities)
}

/// Parses a schema type definition into a list of struct definitions
/// It returns a list because of the inner anonymous types that get generated along the way
/// The last entry in the Vector is the actual entity being requested to parse, I don't care enough right now
/// to fix this retarded API, deal with it. (TODO: fix this)
fn parse_entity(def: SchemaDef, name: String) -> Vec<Entity> {
    match def {
        SchemaDef::Object {
            properties,
            required,
            additional_properties,
            ..
        } => {
            let mut entities = vec![];
            let mut struct_properties: HashMap<String, Field> = HashMap::new();
            let additional_properties = match additional_properties {
                AdditionalProperties::Boolean(true) => Some(FieldType::Object(None)),
                AdditionalProperties::Boolean(false) => None,
                AdditionalProperties::Schema(schema) => {
                    let (field_type, mut new_entities) = parse_schema(*schema);
                    entities.append(&mut new_entities);
                    Some(field_type)
                }
            };
            for (field_name, field_def) in properties.unwrap_or_default() {
                let (field_type, mut new_entities) = parse_schema(field_def);
                let field = Field {
                    optional: !required.contains(&field_name),
                    field_type,
                };
                struct_properties.insert(field_name, field);
                entities.append(&mut new_entities);
            }
            // After parsing all fields build the struct itself
            let struct_def = StructDef {
                properties: struct_properties,
                additional_properties,
            };
            entities.push(Entity {
                name,
                def: EntityDef::Struct(struct_def),
            });
            return entities;
        }
        SchemaDef::AllOf { all_of, .. } => {
            let (all_of_entity_names, mut entities) = parse_combinator_schemas(all_of);
            let all_of_def = Entity { def: EntityDef::AllOf(all_of_entity_names), name };
            entities.push(all_of_def);
            return entities;

        },
        SchemaDef::OneOf {
            one_of,
            discriminant,
            ..
        } => {
            let (variants, mut entities) = parse_combinator_schemas(one_of);
            let one_of_def = Entity { def: EntityDef::OneOf { discriminant, variants }, name };
            entities.push(one_of_def);
            return entities;
        },
        SchemaDef::AnyOf { any_of, .. } => todo!(),
        _ => panic!(
            "Can't parse this type ({:?}) as an entity, only variants allowed: (AllOf, OneOf, AnyOf, Object)", def
        ),
    }
}

/// Entry point for this module, turns a Mapping of `SchemaDef` into a list of `Entity` that a
/// generator can consume to generate code
pub fn parse_schema_def_collection(schema: HashMap<String, SchemaDef>) -> Vec<Entity> {
    schema
        .into_par_iter()
        .flat_map(|(name, schema_def)| parse_entity(schema_def, name))
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::deserializer::SchemaDef;
    use pretty_assertions::assert_eq;
    use std::fs;

    #[test]
    fn test_parse_simple_object_schema() {
        let yaml = r#"
            RequestBase:
              type: object
              properties:
                id:
                  type: string
                kind:
                  type: string
                  const: request
                myDate:
                  type: string
                  format: date-time
                enumProp:
                  type: string
                  enum: [one, two, three]
                deez:
                  type: object
                  properties:
                    value:
                      type: string
                      const: nuts

                refProperty:
                  $ref: '#/components/schemas/RefProperty'
              required:
                - id
                - kind
        "#;
        let parsed_yaml = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
        let entities = parse_schema_def_collection(parsed_yaml);
    }
    #[test]
    fn test_parse_all_of_combinator_schema() {
        let yaml = r#"
            GetUser:
              allOf:
              - $ref: '#/components/schemas/Balls'
              - type: object 
                properties:
                  event:
                    type: string
                    const: deezNuts
                  data:
                    title: GetUserData
                    type: object
                    properties:
                      userId:
                        type: string
                    required:
                      - userId
                required:
                  - data
                  - event
        "#;
        let parsed_yaml = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
        let entities = parse_schema_def_collection(parsed_yaml);
    }
}
