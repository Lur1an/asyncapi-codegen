use lazy_static::lazy_static;
use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    process::Command,
};
use syn::{parse_str, Attribute, Field, Fields, File, Item, ItemEnum, Meta, PathArguments, Type};
use tempfile::TempDir;

/// Joins all source files into one single String file
/// Adds library imports to the top of the file
/// Removes all `crate::` from the source files as they will be in one source and can refer to each
/// other directly
fn join_inputs(inputs: impl Iterator<Item = PathBuf>) -> String {
    let mut output = String::new();
    output.push_str("use serde::{Deserialize, Serialize};\n");
    // output.push_str("use monostate::MustBe;\n");
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

fn find_serde_tag(attrs: &[Attribute]) -> Option<Literal> {
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

/// Remove the field that matches the identifier and return it
fn remove_field(fields: &mut Fields, field_to_remove: &Ident) -> Option<Field> {
    let mut removed_field = None;
    match fields {
        Fields::Named(fields) => {
            // filter fields & edit variable above with ident of removed field
            fields.named = fields
                .named
                .clone()
                .into_iter()
                .filter(|f| {
                    let keep_field = f
                        .ident
                        .as_ref()
                        .map(|identifier| field_to_remove != identifier)
                        .unwrap_or(true);
                    if !keep_field {
                        removed_field = Some(f.clone());
                    }
                    keep_field
                })
                .collect();
        }
        _ => panic!("Only named fields are supported"),
    };
    removed_field
}

fn edit_tag_value(
    enum_item: &mut &mut ItemEnum,
    variant_ident: Ident,
    anonymous_single_value_enum: ItemEnum,
) {
    let target_variant = enum_item
        .variants
        .iter_mut()
        .find(|v| v.ident == variant_ident)
        .unwrap();
    let rename_attribute = &anonymous_single_value_enum
        .variants
        .first()
        .unwrap()
        .attrs
        .first()
        .unwrap();
    let value = if let Meta::List(meta_list) = &rename_attribute.meta {
        let tokens = meta_list
            .tokens
            .clone()
            .into_token_stream()
            .into_iter()
            .take(3)
            .collect::<Vec<_>>();
        match &tokens[..] {
            [TokenTree::Ident(_), TokenTree::Punct(_), TokenTree::Literal(lit)] => {
                Some(lit.clone())
            }
            _ => panic!("Rename attribute must be of the form #[serde(rename = \"value\")]"),
        }
    } else {
        None
    }
    .unwrap();
    println!(
        "The enum item {} needs to have the variant {} renamed to the const fount inside {} which is {:?}",
        enum_item.ident, variant_ident, anonymous_single_value_enum.ident, value
    );
    let target_attribute = target_variant.attrs.iter_mut().next().unwrap();
    println!("Target attribute is {:?}", target_attribute);
    match &mut target_attribute.meta {
        Meta::List(meta_list) => {
            meta_list.tokens = meta_list
                .tokens
                .clone()
                .into_iter()
                .map(|t| {
                    if let TokenTree::Literal(_) = &t {
                        TokenTree::Literal(value.clone())
                    } else {
                        t
                    }
                })
                .collect();
        }
        _ => panic!("Expected a Meta::List here to replace serde rename"),
    }
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

fn enums_first(i1: &Item, _i2: &Item) -> Ordering {
    if let Item::Enum(_) = i1 {
        Ordering::Less
    } else {
        Ordering::Greater
    }
}

pub fn generate_models_from_sources(inputs: impl Iterator<Item = PathBuf>) -> String {
    let joined_file = join_inputs(inputs);
    let mut ast = parse_str::<File>(&joined_file).unwrap();
    let mut enums = Vec::new();
    // The Literal is the name of the field that needs to be removed in all struct variants
    // with name in the `Ident` vector
    let mut duplicate_tags: Vec<(Literal, Vec<Ident>, Ident)> = Vec::new();
    let mut rename_variant_tags = vec![];
    let mut ref_vec = ast.items.iter_mut().collect::<Vec<_>>();
    ref_vec.sort_by(|i1, i2| enums_first(i1, i2));
    for item in ref_vec {
        match item {
            Item::Enum(enum_item) => {
                edit_derive_traits(&mut enum_item.attrs);
                let enum_ident = enum_item.ident.clone();
                // Scanning for a #[serde(tag="value")] is needed because modelina duplicates
                // the "discriminator" field in all nested structs, however deserialization
                // will fail this way because the value of the discriminator field <value> will
                // be consumed by the enum tag resolution and then deserialization of inner
                // struct fails due to missing <value> field from the left over json fields.
                if let Some(serde_tag) = find_serde_tag(&enum_item.attrs) {
                    let variants_to_check = enum_item
                        .variants
                        .iter()
                        .map(|v| v.ident.clone())
                        .collect::<Vec<_>>();
                    duplicate_tags.push((serde_tag, variants_to_check, enum_ident.clone()));
                }
                enums.push(enum_item);
            }
            Item::Struct(struct_item) => {
                edit_derive_traits(&mut struct_item.attrs);
                // Check if this struct is a variant of an enum and if so remove the field that represents
                // the variant as it shouldn't be in the enum
                if let Some((field_name, _, containing_enum_ident)) = duplicate_tags
                    .iter()
                    .find(|(_, variants, _)| variants.contains(&struct_item.ident))
                {
                    let removed_field = remove_field(
                        &mut struct_item.fields,
                        &Ident::new(&field_name.to_string().replace('"', ""), Span::call_site()),
                    );
                    if let Some(anonymous_const) = removed_field {
                        rename_variant_tags.push((
                            anonymous_const,
                            containing_enum_ident.clone(),
                            struct_item.clone(),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    for (anonymous_const_field, containing_enum_ident, struct_item) in rename_variant_tags {
        let type_path = match anonymous_const_field.ty {
            Type::Path(p) => p.path.segments,
            _ => unreachable!(),
        };
        let mut anonymous_const = type_path.first().unwrap().clone();
        while anonymous_const.ident == "Box" {
            if let PathArguments::AngleBracketed(inner_type) = anonymous_const.arguments.clone() {
                match inner_type.args.first().unwrap() {
                    syn::GenericArgument::Type(Type::Path(inner_type)) => {
                        anonymous_const = inner_type.path.segments.first().unwrap().clone();
                    }
                    _ => unimplemented!(),
                }
            }
        }
        let anonymous_inner_enum = enums
            .iter()
            .find(|e| e.ident == anonymous_const.ident)
            .unwrap();
        let anonymous_inner_enum = (*anonymous_inner_enum).clone();
        let enum_item = enums
            .iter_mut()
            .find(|e| e.ident == containing_enum_ident)
            .unwrap();
        edit_tag_value(enum_item, struct_item.ident, anonymous_inner_enum);
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
        let _codegen = generate_models_from_sources(inputs);
    }
    #[test]
    fn test_generate_models_from_spec() {
        let _codegen = generate_models_from_spec(Path::new("./resources/asyncapi-spec.yaml"));
    }
}
