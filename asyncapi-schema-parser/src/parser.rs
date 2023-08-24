use std::collections::HashMap;

use anyhow::anyhow;

/// SchemaProperty can be a reference to a schema by its name or a schema itself
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SchemaDef {
    Object(ObjectDef),
    String(PrimitiveType<String>),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PrimitiveType<T: Eq> {
    Const { const_value: T },
    Enum { enum_values: Vec<T> },
    Basic { format: Option<String> },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ObjectDef {
    pub title: Option<String>,
    pub properties: HashMap<String, ObjectType>,
    pub required: Option<Vec<String>>,
}

pub struct EnumType {
    pub discriminator: Option<String>,
    pub one_of: Vec<ObjectType>,
}

pub struct UnionType {
    pub all_of: Vec<ObjectType>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ObjectType {
    Ref { schema_path: String },
    Def(ObjectDef),
}
