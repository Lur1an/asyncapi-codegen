use lazy_static::lazy_static;
use proc_macro2::{Ident, Literal, TokenStream, TokenTree};
use std::path::PathBuf;
use syn::{Attribute, Meta};

/// Joins all source files into one single String file
/// Adds `use serde::{Deserialize, Serialize};` to the top of the file
/// Removes all `crate::` from the source files as they will be in one source and can refer to each
/// other directly
fn join_inputs(inputs: &[PathBuf]) -> String {
    let mut output = String::new();
    output.push_str("use serde::{Deserialize, Serialize};\n");
    output.push_str("use monostate::MustBe;\n");
    inputs.iter().for_each(|input| {
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

fn edit_derive_traits(attrs: &mut Vec<Attribute>) {
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

fn scan_serde_tag(attrs: &mut Vec<Attribute>) -> Option<Literal> {
    for item in attrs.iter_mut() {
        if let Meta::List(meta_list) = &mut item.meta {
            if meta_list.path.segments.first().unwrap().ident != "serde" {
                continue;
            }
            let tokens = meta_list.tokens.clone().into_iter().collect::<Vec<_>>();
            match &tokens[..] {
                [TokenTree::Ident(ident), TokenTree::Punct(punct), TokenTree::Literal(lit)] => {
                    if ident == "tag" && punct.as_char() == '=' {
                        return Some(lit.clone());
                    }
                }
                _ => {
                    return None;
                }
            }
        }
    }
    None
}

fn replace_single_enum_with_str_const() {}

#[cfg(test)]
mod test {
    use super::*;
    use quote::quote;
    use std::collections::HashMap;
    use syn::{parse_str, File, Item};
    use test_log::test;

    #[test]
    fn dev_test() {
        // let inputs = std::fs::read_dir("./resources/models")
        //     .unwrap()
        //     .into_iter()
        //     .map(Result::unwrap)
        //     .map(|e| e.path())
        //     .collect::<Vec<_>>();
        let inputs = vec![PathBuf::from(
            "./resources/models/one_api_request_payload.rs",
        )];
        let joined_file = join_inputs(&inputs);
        let mut ast = parse_str::<File>(&joined_file).unwrap();
        println!("{:#?}", ast.items);
        let mut structs = HashMap::new();
        let mut enums = HashMap::new();
        for item in ast.items.iter_mut() {
            match item {
                Item::Enum(enum_item) => {
                    edit_derive_traits(&mut enum_item.attrs);
                    // Scanning for a #[serde(tag="value")] is needed because modelina duplicates
                    // the "discriminator" field in all nested structs, however deserialization
                    // will fail this way because the value of the discriminator field <value> will
                    // be consumed by the enum tag resolution and then deserialization of inner
                    // struct fails due to missing <value> field from the left over json fields.
                    let serde_tag = scan_serde_tag(&mut enum_item.attrs);
                    enums.insert(enum_item.ident.clone(), enum_item);
                }
                Item::Struct(struct_item) => {
                    edit_derive_traits(&mut struct_item.attrs);
                    structs.insert(struct_item.ident.clone(), struct_item);
                }
                _ => (),
            }
        }
        // println!("{:#?}", ast.items);
        let new_src = quote! { #ast }.to_string();
        // println!("{}", new_src);
    }
}
