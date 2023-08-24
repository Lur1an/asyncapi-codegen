use std::collections::HashMap;

use anyhow::anyhow;

use crate::deserializer::{SchemaDef, SchemaType};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Schema {
    Object(ObjectSchema),
    String(StringSchema),
    Integer(IntSchema),
    SubSchema { name: String },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ObjectSchema {
    pub additional_properties: bool,
    pub properties: HashMap<String, Schema>,
    pub discriminator: Option<String>,
    pub required: Option<Vec<String>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum StringSchema {
    Const(ConstSchema<String>),
    Enum(EnumSchema<String>),
    Basic,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConstSchema<T> {
    pub const_value: T,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EnumSchema<T> {
    pub enum_values: Vec<T>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum IntSchema {
    Const(ConstSchema<i64>),
    Enum(EnumSchema<i64>),
    Basic,
}

fn parse_string_schema(schema: SchemaDef) -> anyhow::Result<StringSchema> {
    if schema.const_value.is_some() && schema.enum_values.is_some() {
        return Err(anyhow!(
            "Schema cannot have both const and enum set at the same time"
        ));
    }
    if let Some(const_value) = schema.const_value {
        return Ok(StringSchema::Const(ConstSchema { const_value }));
    }
    if let Some(enum_values) = schema.enum_values {
        return Ok(StringSchema::Enum(EnumSchema { enum_values }));
    }
    Ok(StringSchema::Basic)
}

pub fn parse_schema(raw_schema: HashMap<String, SchemaDef>) -> anyhow::Result<Vec<Schema>> {
    let mut schemas: HashMap<String, Schema> = HashMap::new();
    for (name, schema) in raw_schema {
        match schema.schema_type {
            Some(SchemaType::Object) => todo!(),
            Some(SchemaType::String) => {
                parse_string_schema(schema)?;
            }
            Some(SchemaType::Integer) => todo!(),
            Some(SchemaType::Number) => todo!(),
            None => todo!(),
        }
    }
    todo!()
}
