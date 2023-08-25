use std::{collections::HashMap, sync::atomic::AtomicU32};

use lazy_static::lazy_static;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::deserializer::{Schema, SchemaDef, SchemaType};

/// SchemaProperty can be a reference to a schema by its name or a schema itself
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FieldType {
    Entity { entity_name: String },
    String(PrimitiveType<String>),
    Integer(PrimitiveType<i64>),
    Object,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Field {
    pub optional: bool,
    pub field_type: FieldType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrimitiveType<T> {
    Const(T),
    Enum(Vec<T>),
    Basic { format: Option<String> },
}

impl<T> Default for PrimitiveType<T> {
    fn default() -> Self {
        PrimitiveType::Basic { format: None }
    }
}

/// The definition for a Struct type
/// All that is needed is a map of properties
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructDef {
    pub properties: HashMap<String, Field>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntityDef {
    /// A simple definition for a Class-like entity
    Struct(StructDef),
    /// A Collection of Variants and an Optional discriminant
    OneOf {
        discriminant: Option<String>,
        variants: Vec<String>,
    },
    /// AllOf is the inheritance operator, all structs that are inherited are referenced by name
    /// And resolved at generation time
    AllOf(Vec<String>),
}

/// An entity is any kind of type that needs to be generated in the result code
/// It always has a name and a definition (Struct or Enum)
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entity {
    pub name: String,
    pub def: EntityDef,
}

lazy_static! {
    static ref ANONYMOUS_STRUCT_COUNT: AtomicU32 = AtomicU32::new(1);
}

/// Parses a Schema element into a FieldType and a list of Entities that might be correlated to the
/// field (e.g. anonymous structs that are nested below a field, which will need to be generated)
fn parse_schema(schema: Schema) -> (FieldType, Vec<Entity>) {
    match schema {
        Schema::Ref(schema_ref) => {
            let name = schema_ref.get_schema_name().to_string();
            (FieldType::Entity { entity_name: name }, vec![])
        }
        Schema::Def(schema_def) => match schema_def.schema_type {
            Some(SchemaType::Object) => (FieldType::Object, vec![]),
            Some(SchemaType::String) => (FieldType::String(PrimitiveType::default()), vec![]),
            Some(SchemaType::Integer) => (FieldType::Integer(PrimitiveType::default()), vec![]),
            Some(SchemaType::Number) => (FieldType::Integer(PrimitiveType::default()), vec![]),
            None => {
                let inner_schema_name = format!(
                    "AnonymousEntity{}",
                    ANONYMOUS_STRUCT_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                );
                (
                    FieldType::Entity {
                        entity_name: inner_schema_name.clone(),
                    },
                    parse_entity(schema_def, inner_schema_name),
                )
            }
        },
    }
}

/// Parses a schema type definition into a list of struct definitions
/// It returns a list because of the inner anonymous types that get generated along the way
fn parse_entity(def: SchemaDef, name: String) -> Vec<Entity> {
    if def.schema_type.is_some() {
        panic!("Expected a schema definition, not a primitive type");
    }

    if let Some(properties) = def.properties {
        if def.all_of.is_some() || def.one_of.is_some() {
            panic!("Schema Definition with properties shouldn't have allOf or oneOf combinators");
        }
        let mut entities = vec![];
        let mut struct_properties: HashMap<String, Field> = HashMap::new();
        for (field_name, field_def) in properties {
            let (field_type, mut new_entities) = parse_schema(field_def);
            let field = Field {
                optional: def
                    .required
                    .as_ref()
                    .map_or(true, |required| !required.contains(&field_name)),
                field_type,
            };
            struct_properties.insert(field_name, field);
            entities.append(&mut new_entities);
        }
        // After parsing all fields build the struct itself
        let struct_def = StructDef {
            properties: struct_properties,
        };
        entities.push(Entity {
            name,
            def: EntityDef::Struct(struct_def),
        });
        return entities;
    }
    if let Some(all_of) = def.all_of {
        if def.one_of.is_some() {
            panic!("Schema already has a oneOf combinator, can't have allOf as well");
        }
        let mut composing_entities = vec![];
        let mut entities = vec![];
        all_of
            .into_iter()
            .map(|schema| parse_schema(schema))
            .for_each(|(parsed_field, parsed_entities)| {
                let entity_name = match parsed_field {
                    FieldType::Entity { entity_name } => entity_name,
                    _ => panic!("AllOf can only contain entities, no primitive types"),
                };
                composing_entities.push(entity_name);
                entities.extend(parsed_entities);
            });
        entities.push(Entity {
            name,
            def: EntityDef::AllOf(composing_entities),
        });
        return entities;
    }
    if let Some(one_of) = def.one_of {
        let mut composing_entities = vec![];
        let mut entities = vec![];
        one_of
            .into_iter()
            .map(|schema| parse_schema(schema))
            .for_each(|(parsed_field, parsed_entities)| {
                let entity_name = match parsed_field {
                    FieldType::Entity { entity_name } => entity_name,
                    _ => panic!("AllOf can only contain entities, no primitive types"),
                };
                composing_entities.push(entity_name);
                entities.extend(parsed_entities);
            });
        entities.push(Entity {
            name,
            def: EntityDef::OneOf {
                discriminant: def.discriminant,
                variants: composing_entities,
            },
        });
        return entities;
    }
    todo!()
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
    use pretty_assertions::assert_eq;

    use crate::deserializer::SchemaDef;

    use super::*;
    use std::fs;

    #[test]
    fn test_parse_simple_object_schema() {
        let yaml = r#"
            RequestBase:
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
        assert_eq!(entities.len(), 2);
        let request_base = entities.iter().find(|e| e.name == "RequestBase").unwrap();
        assert_eq!(request_base.name, "RequestBase");
        let struct_def = match &request_base.def {
            EntityDef::Struct(struct_def) => struct_def,
            _ => panic!("Expected a struct definition"),
        };
        assert_eq!(struct_def.properties.len(), 6);
        assert!(!struct_def.properties["kind"].optional);
    }
    #[test]
    fn test_parse_all_of_combinator_schema() {
        let yaml = r#"
            GetUser:
              description: TODO
              discriminant: balls
              allOf:
              - $ref: '#/components/schemas/RequestBase'
              - properties:
                  event:
                    type: string
                    const: deezNuts
                  data:
                    title: GetUserData
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
        assert_eq!(entities.len(), 3);
        let get_user = entities.iter().find(|e| e.name == "GetUser").unwrap();
        assert_eq!(get_user.name, "GetUser");
        let all_of = match &get_user.def {
            EntityDef::AllOf(all_of) => all_of,
            _ => panic!("Expected an AllOf definition"),
        };
        assert_eq!(all_of.len(), 2);
    }
}
