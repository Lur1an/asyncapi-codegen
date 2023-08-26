use quote::quote;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::parser::{Entity, EntityDef, FieldType, PrimitiveType};

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
                    FieldType::Simple {
                        type_identifier: entity_name,
                    } => {
                        if field.optional {
                            quote! {
                                #name: Option<#entity_name>
                            }
                        } else {
                            quote! {
                                #name: #entity_name
                            }
                        }
                    }
                    FieldType::String(f) => match f {
                        PrimitiveType::Const(const_field) => todo!(),
                        PrimitiveType::Enum(enum_field) => todo!(),
                        PrimitiveType::Basic { format } => {
                            if field.optional {
                                quote! {
                                    #name: Option<String>
                                }
                            } else {
                                quote! {
                                    #name: String
                                }
                            }
                        }
                    },
                    FieldType::Integer(_) => todo!(),
                    FieldType::Object => todo!(),
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
    };
    code.to_string()
}

#[cfg(test)]
mod test {
    use crate::parser::{Field, PrimitiveType, StructDef};

    use super::*;

    #[test]
    fn test_generate_struct() {
        let struct_def = EntityDef::Struct(StructDef {
            properties: vec![(
                "fieldName".to_string(),
                Field {
                    field_type: FieldType::Simple {
                        type_identifier: "FieldEntityName".to_string(),
                    },
                    optional: false,
                },
            )]
            .into_iter()
            .collect(),
        });
        let entity = Entity {
            name: "EntityName".to_string(),
            def: struct_def,
        };
        let code = generate_entity(entity);
        println!("{}", code);
    }
}
