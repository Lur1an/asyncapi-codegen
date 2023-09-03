use std::collections::HashMap;

use monostate::MustBe;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    pub schema_path: String,
}

impl SchemaRef {
    pub fn get_schema_name(&self) -> &str {
        self.schema_path
            .split('/')
            .last()
            .expect("Incorrect Ref Path")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SchemaType {
    Object,
    String,
    Integer,
    Number,
    Array,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    Int32,
    Int64,
    Float,
    Double,
    Byte,
    Binary,
    Date,
    Uuid,
    #[serde(rename = "date-time")]
    DateTime,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum PrimitiveType<T> {
    Const {
        #[serde(rename = "const")]
        const_value: T,
    },
    Enum {
        #[serde(rename = "enum")]
        enum_values: Vec<T>,
    },
    Basic {
        format: Option<Format>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum AdditionalProperties {
    Boolean(bool),
    Schema(Box<Schema>),
}

impl Default for AdditionalProperties {
    fn default() -> Self {
        AdditionalProperties::Boolean(false)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum SchemaDef {
    Object {
        title: Option<String>,
        #[serde(rename = "type")]
        schema_type: MustBe!("object"),
        #[serde(default)]
        #[serde(rename = "additionalProperties")]
        additional_properties: AdditionalProperties,
        properties: Option<HashMap<String, Schema>>,
        #[serde(default)]
        required: Vec<String>,
    },
    String {
        #[serde(rename = "type")]
        schema_type: MustBe!("string"),
        #[serde(flatten)]
        type_def: PrimitiveType<String>,
    },
    Integer {
        #[serde(rename = "type")]
        schema_type: MustBe!("integer"),
        #[serde(flatten)]
        type_def: PrimitiveType<i64>,
    },
    Boolean {
        #[serde(rename = "type")]
        schema_type: MustBe!("boolean"),
    },
    Number {
        #[serde(rename = "type")]
        schema_type: MustBe!("number"),
        #[serde(flatten)]
        type_def: PrimitiveType<f64>,
    },
    Array {
        #[serde(rename = "type")]
        schema_type: MustBe!("array"),
        items: Option<Box<Schema>>,
    },
    Tuple {
        #[serde(rename = "type")]
        schema_type: MustBe!("array"),
        items: MustBe!(false),
        #[serde(rename = "prefixItems")]
        prefix_items: Vec<Schema>,
    },
    AllOf {
        title: Option<String>,
        #[serde(rename = "allOf")]
        all_of: Vec<Schema>,
    },
    OneOf {
        title: Option<String>,
        #[serde(rename = "oneOf")]
        one_of: Vec<Schema>,
        discriminator: Option<String>,
    },
    AnyOf {
        title: Option<String>,
        #[serde(rename = "anyOf")]
        any_of: Vec<Schema>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
/// A Schema can either be a $ref to another Schema or a Definition of a Schema.
/// This deserializer assumes all top-level types are `SchemaDef`
pub enum Schema {
    Ref(SchemaRef),
    Def(SchemaDef),
}

#[cfg(test)]
mod test {
    use std::fs;

    use serde_yaml::Value;

    use super::*;

    #[test]
    fn test_parse_complex_schema() {
        let content = include_str!("../resources/asyncapi.yaml");
        let parsed_yaml = serde_yaml::from_str::<Value>(&content).unwrap();
        let _parsed_schema = serde_yaml::from_value::<HashMap<String, SchemaDef>>(
            parsed_yaml["components"]["schemas"].clone(),
        )
        .unwrap();
    }

    #[test]
    fn test_parse_object_schema() {
        let yaml = r#"
            GetUser:
              type: object
              additionalProperties:
                $ref: '#/components/schemas/SomeOtherEntity'
              
        "#;
        let _ = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
    }

    #[test]
    fn test_parse_schema_combinators() {
        let yaml = r#"
            GetUser:
              description: TODO
              allOf:
              - $ref: '#/components/schemas/RequestBase'
              - type: object
                additionalProperties: false
                properties:
                  event:
                    type: string
                    const: deezNuts
                  arrayType:
                    type: array
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
        let _ = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
    }

    #[test]
    fn test_parse_string_basic() {
        let yaml = r#"
            id:
                type: string
                description: "correlation id to match request and response"
        "#;
        let _ = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
    }

    #[test]
    fn test_parse_string_const() {
        let yaml = r#"
            deez:
                type: string
                const: nuts
        "#;
        let _ = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
    }
}
