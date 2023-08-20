use lazy_static::lazy_static;
use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};
use syn::{parse_str, Attribute, Fields, File, Item, Meta};
use tempfile::TempDir;

/// Joins all source files into one single String file
/// Adds library imports to the top of the file
/// Removes all `crate::` from the source files as they will be in one source and can refer to each
/// other directly
fn join_inputs(inputs: impl Iterator<Item = PathBuf>) -> String {
    let mut output = String::new();
    output.push_str("use serde::{Deserialize, Serialize};\n");
    output.push_str("use monostate::MustBe;\n");
    inputs.for_each(|input| {
        let content = std::fs::read_to_string(input).unwrap();
        let content = content.replace("crate::", "");
        output.push_str(&content);
        output.push('\n');
    });
    output
}

fn should_remove_trait(ident: &Ident) -> bool {
    lazy_static! {
        static ref BUGGY_TRAITS: [&'static str; 6] =
            ["Ord", "PartialOrd", "PartialEq", "Eq", "Hash", "Copy"];
    }
    BUGGY_TRAITS.contains(&ident.to_string().as_str())
}

/// Removes buggy traits from the derive macro like Eq, Hash, Copy, Ord, etc.
/// Will be configurable in the future or will remove all traits and then just insert new ones
/// depending on config
fn remove_buggy_traits(
    item: impl IntoIterator<Item = TokenTree>,
) -> impl Iterator<Item = TokenTree> {
    let mut skip_next = false;
    item.into_iter().filter(move |t| {
        if skip_next {
            skip_next = false;
            return false;
        }
        match t {
            TokenTree::Ident(ident) => {
                if should_remove_trait(ident) {
                    skip_next = true;
                    return false;
                }
                true
            }
            _ => true,
        }
    })
}

fn edit_derive_traits(attrs: &mut [Attribute]) {
    attrs.iter_mut().for_each(|item| {
        if let Meta::List(meta_list) = &mut item.meta {
            if meta_list.path.segments.first().unwrap().ident != "derive" {
                return;
            }
            let new_tokens = remove_buggy_traits(meta_list.tokens.clone());
            meta_list.tokens = new_tokens.collect::<TokenStream>();
        }
    });
}

fn scan_serde_tag(attrs: &[Attribute]) -> Option<Literal> {
    for item in attrs.iter() {
        if let Meta::List(meta_list) = &item.meta {
            if meta_list.path.segments.first().unwrap().ident != "serde" {
                continue;
            }
            let tokens = meta_list
                .tokens
                .clone()
                .into_token_stream()
                .into_iter()
                .take(3)
                .collect::<Vec<_>>();
            match &tokens[..] {
                [TokenTree::Ident(ident), TokenTree::Punct(punct), TokenTree::Literal(lit)] => {
                    if ident == "tag" && punct.as_char() == '=' {
                        return Some(lit.clone());
                    }
                }
                _ => {
                    continue;
                }
            }
        }
    }
    None
}

/// Removes all field that match the given identifiers
fn remove_fields_named(fields: &mut Fields, fields_to_remove: &[Ident]) {
    match fields {
        Fields::Named(fields) => {
            fields.named = fields
                .named
                .clone()
                .into_iter()
                .filter(|f| {
                    let keep_field = f
                        .ident
                        .as_ref()
                        .map(|identifier| !fields_to_remove.contains(identifier))
                        .unwrap_or(true);
                    keep_field
                })
                .collect();
        }
        _ => panic!("Only named fields are supported"),
    };
}
pub fn generate_models_from_spec(spec_path: &Path) -> String {
    let temp_dir = TempDir::new().unwrap();
    let models_path = temp_dir.path();
    let args = [
        "generate",
        "models",
        "rust",
        spec_path.to_str().unwrap(),
        "-o",
        models_path.to_str().unwrap(),
    ];
    let _ = Command::new("asyncapi").args(&args).output().unwrap();
    let inputs = std::fs::read_dir(models_path)
        .unwrap()
        .map(Result::unwrap)
        .map(|e| e.path());
    let codegen = generate_models_from_sources(inputs);
    codegen
}

pub fn generate_models_from_sources(inputs: impl Iterator<Item = PathBuf>) -> String {
    let joined_file = join_inputs(inputs);
    let mut ast = parse_str::<File>(&joined_file).unwrap();
    let mut enums = HashMap::new();
    // The Literal is the name of the field that needs to be removed in all struct variants
    // with name in the `Ident` vector
    let mut duplicate_tags: Vec<(Literal, Vec<Ident>)> = Vec::new();
    for enum_item in ast.items.iter_mut().filter_map(|i| {
        if let Item::Enum(enum_item) = i {
            Some(enum_item)
        } else {
            None
        }
    }) {
        edit_derive_traits(&mut enum_item.attrs);
        // Scanning for a #[serde(tag="value")] is needed because modelina duplicates
        // the "discriminator" field in all nested structs, however deserialization
        // will fail this way because the value of the discriminator field <value> will
        // be consumed by the enum tag resolution and then deserialization of inner
        // struct fails due to missing <value> field from the left over json fields.
        if let Some(serde_tag) = scan_serde_tag(&enum_item.attrs) {
            let variants_to_check = enum_item
                .variants
                .iter()
                .map(|v| v.ident.clone())
                .collect::<Vec<_>>();
            duplicate_tags.push((serde_tag, variants_to_check));
        }
        enums.insert(enum_item.ident.clone(), enum_item);
    }

    for struct_item in ast.items.iter_mut().filter_map(|i| {
        if let Item::Struct(struct_item) = i {
            Some(struct_item)
        } else {
            None
        }
    }) {
        edit_derive_traits(&mut struct_item.attrs);
        if let Some((field_name, _)) = duplicate_tags
            .iter()
            .find(|(_, variants)| variants.contains(&struct_item.ident))
        {
            remove_fields_named(
                &mut struct_item.fields,
                &[Ident::new(
                    &field_name.to_string().replace('"', ""),
                    Span::call_site(),
                )],
            );
        }
    }
    quote! { #ast }.to_string()
}

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;

    #[test]
    fn test_generate_models() {
        let inputs = std::fs::read_dir("./resources/models")
            .unwrap()
            .map(Result::unwrap)
            .map(|e| e.path());
        let codegen = generate_models_from_sources(inputs);
        log::info!("{}", codegen);
    }
    #[test]
    fn test_generate_models_from_spec() {
        let codegen = generate_models_from_spec(Path::new("./resources/asyncapi-spec.yaml"));
        log::info!("{}", codegen);
    }
}
