use quote::quote;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::parser::Entity;

pub fn generate_code(entities: Vec<Entity>) -> String {
    let code = entities.into_par_iter().map(generate_entity);
    todo!()
}

fn generate_entity(entity: Entity) -> String {
    let identifier = entity.name;
    match entity.def {
        crate::parser::EntityDef::Struct(struct_def) => todo!(),
        crate::parser::EntityDef::OneOf {
            discriminant,
            variants,
        } => todo!(),
        crate::parser::EntityDef::AllOf(all_of) => todo!(),
    };
    todo!()
}
