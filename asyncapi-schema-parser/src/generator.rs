use quote::quote;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::parser::{Entity, EntityDef, FieldType};

pub fn generate_code(entities: Vec<Entity>) -> String {
    let code = entities.into_par_iter().map(generate_entity);
    todo!()
}

fn snake_case(s: &str) -> &str {
    todo!()
}

fn generate_entity(entity: Entity) -> String {
    let identifier = entity.name;
    let code = match entity.def {
        EntityDef::Struct(struct_def) => {
            let fields = struct_def
                .properties
                .into_iter()
                .map(|(name, field)| match field.field_type {
                    FieldType::Named(type_identifier) => {
                        if field.optional {
                            quote! {
                                #name: Option<#type_identifier>
                            }
                        } else {
                            quote! {
                                #name: #type_identifier
                            }
                        }
                    }
                    _ => todo!(),
                })
                .collect::<Vec<_>>();
            quote! {
                struct #identifier {
                    #(#fields),*
                }
            }
        }

        EntityDef::OneOf {
            discriminant,
            variants,
        } => todo!(),
        EntityDef::AllOf(all_of) => todo!(),
        EntityDef::Enum(_) => todo!(),
    };
    code.to_string()
}

#[cfg(test)]
mod test {
    use crate::parser::{Field, StructDef};

    use super::*;

    #[test]
    fn test_generate_struct() {
        let struct_def = EntityDef::Struct(StructDef {
            properties: vec![(
                "fieldName".to_string(),
                Field {
                    field_type: FieldType::Named("FieldEntityName".to_string()),
                    optional: false,
                },
            )]
            .into_iter()
            .collect(),
            additional_properties: None,
        });
        let entity = Entity {
            name: "EntityName".to_string(),
            def: struct_def,
        };
        let code = generate_entity(entity);
        println!("{}", code);
    }
}
