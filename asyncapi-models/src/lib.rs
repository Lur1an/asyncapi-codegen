use std::collections::HashMap;

use deserializer::SchemaDef;

mod deserializer;
mod generator;
pub(crate) mod parser;

pub fn generate_rust(input: &str) -> String {
    let input = serde_yaml::from_str::<serde_yaml::Value>(input).unwrap();
    let input = serde_yaml::from_value::<HashMap<String, SchemaDef>>(
        input["components"]["schemas"].clone(),
    )
    .unwrap();
    let entities = parser::parse_schema_def_collection(input);
    let code = generator::generate_rust(entities);
    code
}
