use super::snake_case;
use proc_macro2::TokenStream;
use quote::quote;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::parser::{Entity, EntityDef, EnumDef, FieldType, Primitive, StructDef};

pub fn generate_code(entities: Vec<Entity>) -> String {
    let code = entities
        .into_par_iter()
        .map(generate_entity)
        .collect::<Vec<_>>();
    code.join("\n")
}

fn expand_field_type(field_type: FieldType) -> String {
    match field_type {
        FieldType::Named(t) => t,
        FieldType::Array(Some(item_type)) => format!("Vec<{}>", expand_field_type(*item_type)),
        FieldType::Array(None) => "Vec<serde_json::Value>".into(),
        FieldType::Object(Some(value_type)) => {
            format!(
                "std::collections::HashMap<String, {}>",
                expand_field_type(*value_type)
            )
        }
        FieldType::Object(None) => "serde_json::Value".into(),
        FieldType::Tuple(tuple_types) => {
            let tuple_types = tuple_types
                .into_iter()
                .map(expand_field_type)
                .collect::<Vec<_>>();
            format!("({})", tuple_types.join(", "))
        }
        FieldType::Simple(primitive) => match primitive {
            Primitive::String => "String".into(),
            Primitive::Int => "i32".into(),
            Primitive::Double => "f64".into(),
            Primitive::Bool => "bool".into(),
            Primitive::Long => "i64".into(),
            Primitive::Float => "f32".into(),
            Primitive::Uuid => "uuid::Uuid".into(),
            Primitive::Bytes => "Vec<u8>".into(),
            Primitive::U32 => "u32".into(),
            Primitive::U64 => "u64".into(),
        },
        FieldType::Const(primitive, value) => match primitive {
            Primitive::String => format!("monostate::MustBe!(\"{}\")", value),
            Primitive::Int => format!("monostate::MustBe!({})", value),
            Primitive::Double => format!("monostate::MustBe!({})", value),
            Primitive::Bool => format!("monostate::MustBe!({})", value),
            Primitive::Long => format!("monostate::MustBe!({})", value),
            Primitive::Float => format!("monostate::MustBe!({})", value),
            Primitive::U32 => format!("monostate::MustBe!({})", value),
            Primitive::U64 => format!("monostate::MustBe!({})", value),
            Primitive::Uuid => todo!(),
            Primitive::Bytes => todo!(),
        },
    }
}

fn generate_entity(entity: Entity) -> String {
    let identifier: TokenStream = entity.name.parse().unwrap();
    let code = match entity.def {
        EntityDef::Struct(StructDef {
            properties,
            additional_properties,
        }) => {
            let mut fields = properties
                .into_iter()
                .map(|(name, field)| {
                    let field_type: TokenStream =
                        expand_field_type(field.field_type).parse().unwrap();
                    let field_name: TokenStream = snake_case(&name).parse().unwrap();
                    if field.optional {
                        quote! {
                            #[serde(rename = #name)]
                            pub #field_name: Option<#field_type>
                        }
                    } else {
                        quote! {
                            #[serde(rename = #name)]
                            pub #field_name: #field_type
                        }
                    }
                })
                .collect::<Vec<_>>();
            if let Some(additional_properties) = additional_properties {
                let field_type = expand_field_type(additional_properties)
                    .parse::<TokenStream>()
                    .unwrap();
                fields.push(quote! {
                    #[serde(flatten)]
                    pub additional_properties: std::collections::HashMap<String, #field_type>
                })
            }

            quote! {
                #[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
                pub struct #identifier {
                    #(#fields),*
                }
            }
        }

        EntityDef::OneOf {
            discriminant,
            variants,
        } => {
            let variants = variants.into_iter().map(|variant| {
                let variant_name: TokenStream = variant.parse().unwrap();
                quote! {
                    #variant_name(#variant_name)
                }
            });
            if let Some(discriminant) = discriminant {
                quote! {
                    #[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
                    #[serde(tag = #discriminant)]
                    pub enum #identifier {
                        #(#variants),*
                    }

                }
            } else {
                quote! {
                    #[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
                    #[serde(untagged)]
                    pub enum #identifier {
                        #(#variants),*
                    }
                }
            }
        }
        EntityDef::AllOf(all_of) => {
            let flattened_structs = all_of.into_iter().map(|entity| {
                let field_name = snake_case(&entity).parse::<TokenStream>().unwrap();
                let field_type = entity.parse::<TokenStream>().unwrap();
                quote! {
                    #[serde(flatten)]
                    pub #field_name: #field_type
                }
            });
            quote! {
                #[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
                pub struct #identifier {
                    #(#flattened_structs),*
                }
            }
        }
        EntityDef::Enum(EnumDef { values }) => {
            let variants = values.into_iter().map(|value| {
                let value: TokenStream = value.parse().unwrap();
                quote! {
                    #value
                }
            });
            quote! {
                #[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
                pub enum #identifier {
                    #(#variants),*
                }
            }
        }
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
            properties: vec![
                (
                    "fieldName".to_string(),
                    Field {
                        field_type: FieldType::Named("FieldEntityName".to_string()),
                        optional: true,
                    },
                ),
                (
                    "constField".to_string(),
                    Field {
                        field_type: FieldType::Const(Primitive::String, "constValue".to_string()),
                        optional: false,
                    },
                ),
            ]
            .into_iter()
            .collect(),
            additional_properties: Some(FieldType::Array(None)),
        });
        let entity = Entity {
            name: "StructEntity".to_string(),
            def: struct_def,
        };
        let code = generate_entity(entity);
        println!("{}", code);
        assert!(code.contains("pub struct StructEntity"));
        assert!(code
            .replace(" ", "")
            .contains("field_name:Option<FieldEntityName>"));
        assert!(code
            .replace(" ", "")
            .contains("const_field:monostate::MustBe!(\"constValue\")"));
    }

    #[test]
    fn test_generate_tagged_enum() {
        let enum_def = EntityDef::OneOf {
            discriminant: Some("type".to_string()),
            variants: vec!["Variant1".to_string(), "Variant2".to_string()],
        };
        let entity = Entity {
            name: "EnumEntity".to_string(),
            def: enum_def,
        };
        let code = generate_entity(entity);
        println!("{}", code);
        assert!(code.contains("pub enum EnumEntity"));
        assert!(code.replace(" ", "").contains("#[serde(tag=\"type\")]"));
    }
}
