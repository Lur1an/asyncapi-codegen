use deserializer::SchemaDef;
use std::collections::HashMap;

pub mod deserializer;
mod generator;
pub(crate) mod parser;

pub fn generate_rust(input: HashMap<String, SchemaDef>) -> String {
    let entities = parser::parse_schema_def_collection(input);
    generator::generate_rust(entities)
}
