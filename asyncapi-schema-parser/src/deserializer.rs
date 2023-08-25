use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
pub struct SchemaRef {
    #[serde(rename = "$ref")]
    pub schema_path: String,
}

impl SchemaRef {
    pub fn get_schema_name(&self) -> &str {
        self.schema_path
            .split("/")
            .last()
            .expect("Incorrect Ref Path")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum SchemaType {
    Object,
    String,
    Integer,
    Number,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    Int32,
    Int64,
    Float,
    Double,
    Byte,
    Binary,
    Date,
    #[serde(rename = "date-time")]
    DateTime,
}

/// SchemaProperty can be a reference to a schema by its name or a schema itself
#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SchemaDef {
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub schema_type: Option<SchemaType>,
    #[serde(rename = "const")]
    pub const_value: Option<String>,
    pub format: Option<Format>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<String>>,
    pub one_of: Option<Vec<Schema>>,
    pub all_of: Option<Vec<Schema>>,
    pub any_of: Option<Vec<Schema>>,
    pub required: Option<Vec<String>>,
    pub properties: Option<HashMap<String, Schema>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Eq, PartialEq)]
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
        let content = fs::read_to_string("./resources/asyncapi.yaml").unwrap();
        let parsed_yaml = serde_yaml::from_str::<Value>(&content).unwrap();
        let parsed_schema = serde_yaml::from_value::<HashMap<String, SchemaDef>>(
            parsed_yaml["components"]["schemas"].clone(),
        )
        .unwrap();
    }

    #[test]
    fn test_parse_object_schema() {
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

    #[test]
    fn test_parse_schema_combinators() {
        let yaml = r#"
            GetUser:
              type: object
              description: TODO
              allOf:
              - $ref: '#/components/schemas/RequestBase'
              - type: object
                properties:
                  event:
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
    }

    #[test]
    fn test_parse_string_basic() {
        let yaml = r#"
            id:
                type: string
                description: "correlation id to match request and response"
        "#;
        let parsed_yaml = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
    }

    #[test]
    fn test_parse_string_const() {
        let yaml = r#"
            deez:
                type: string
                const: nuts
        "#;
        let _parsed_yaml = serde_yaml::from_str::<HashMap<String, SchemaDef>>(yaml).unwrap();
    }
}
