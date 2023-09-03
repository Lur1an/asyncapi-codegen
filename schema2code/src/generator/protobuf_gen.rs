use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::parser::Entity;

pub fn generate_code(entities: Vec<Entity>) -> String {
    let code = entities
        .into_par_iter()
        .map(generate_entity)
        .collect::<Vec<_>>();
    code.join("\n")
}

fn generate_entity(entity: Entity) -> String {
    let identifier = entity.name;
    let content = "";
    format!(
        r#"
message {identifier} {{
    {content}
}}
    "#,
    )
}
