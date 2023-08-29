mod rust_gen;
use crate::parser::Entity;

pub fn generate_rust(entities: Vec<Entity>) -> String {
    rust_gen::generate_code(entities)
}
