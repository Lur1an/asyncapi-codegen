use std::{cell::Cell, collections::HashMap, sync::atomic::AtomicU32};

use anyhow::anyhow;
use lazy_static::lazy_static;

use crate::deserializer::{Schema, SchemaDef, SchemaType};

/// SchemaProperty can be a reference to a schema by its name or a schema itself
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FieldType {
    Object { struct_def_name: String },
    String(PrimitiveType<String>),
    Integer(PrimitiveType<i64>),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StructDef {
    pub name: String,
    pub properties: HashMap<String, Field>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub discriminant: Option<String>,
    pub variants: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntityDef {
    Struct(StructDef),
    Enum(EnumDef),
}

lazy_static! {
    static ref ANONYMOUS_STRUCT_COUNT: AtomicU32 = AtomicU32::new(1);
}

fn parse_field_def(def: Schema) -> (FieldType, Vec<EntityDef>) {
    match def {
        Schema::Ref(schema_ref) => {
            let name = schema_ref.get_schema_name().to_string();
            (
                FieldType::Object {
                    struct_def_name: name,
                },
                vec![],
            )
        }
        Schema::Def(_) => todo!(),
    }
}

/// Parses a schema definition into a list of struct definitions
/// It returns a list because of the inner anonymous types that get generated along the way
fn parse_root_schema_def(def: SchemaDef, name: String) -> Vec<EntityDef> {
    // let name = def.title.or(name).unwrap_or_else(|| {
    //     format!(
    //         "AnonymousStruct{}",
    //         ANONYMOUS_STRUCT_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    //     )
    // });
    let mut structs = vec![];
    let mut struct_properties: HashMap<String, Field> = HashMap::new();
    if let Some(properties) = def.properties {
        for (field_name, field_def) in properties {
            let (field_type, mut parsed_structs) = parse_field_def(field_def);
            let field = Field {
                optional: false,
                field_type,
            };
            struct_properties.insert(field_name, field);
            structs.append(&mut parsed_structs);
        }
    }

    todo!()
}

pub fn parse_schema(schema: HashMap<String, SchemaDef>) -> Vec<StructDef> {
    let mut structs: HashMap<String, StructDef> = HashMap::new();
    for (name, schema_def) in schema {
        let parsed_structs = parse_root_schema_def(schema_def, name);
        for struct_def in parsed_structs {
            structs.insert(struct_def.name.clone(), struct_def);
        }
    }
    todo!()
}

#[cfg(test)]
mod test {
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
                refProperty:
                  $ref: '#/components/schemas/RefProperty'
              required:
                - id
                - kind
        "#;
        let parsed_yaml = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
    }
}
