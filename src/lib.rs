use std::path::PathBuf;

use syn::{Attribute, Item, Meta};

fn join_inputs(inputs: &[PathBuf]) -> String {
    let mut output = String::new();
    output.push_str("use serde::{Deserialize, Serialize};\n");
    inputs.iter().for_each(|input| {
        let content = std::fs::read_to_string(input).unwrap();
        let content = content.replace("crate::", "");
        output.push_str(&content);
        output.push('\n');
    });
    output
}

fn get_meta(attrs: &[Attribute]) -> Meta {
    todo!()
}

fn edit_derive_traits(item: &mut Vec<Attribute>) {
    let attrs = log::info!("Editing derive traits");
}

#[cfg(test)]
mod test {
    use quote::{ToTokens, __private::Span, quote};
    use syn::{parse_str, File, Ident, Item};
    use test_log::test;

    use super::*;

    #[test]
    fn test_join_inputs() {
        // let inputs = std::fs::read_dir("./resources/models")
        //     .unwrap()
        //     .into_iter()
        //     .map(Result::unwrap)
        //     .map(|e| e.path())
        //     .collect::<Vec<_>>();
        let inputs = vec![PathBuf::from("./resources/models/anonymous_schema8.rs")];
        let joined_file = join_inputs(&inputs);
        let mut ast = parse_str::<File>(&joined_file).unwrap();
        println!("{:#?}", ast.items);
        for item in ast.items.iter_mut() {
            match item {
                Item::Const(_) => (),
                Item::Enum(enum_item) => {
                    edit_derive_traits(&mut enum_item.attrs);
                    enum_item.ident = Ident::new("DeezNutsOnYourChin", Span::call_site());
                }
                Item::ExternCrate(_) => (),
                Item::Fn(_) => (),
                Item::ForeignMod(_) => (),
                Item::Impl(_) => (),
                Item::Macro(_) => (),
                Item::Mod(_) => (),
                Item::Static(_) => (),
                Item::Struct(_) => (),
                Item::Trait(_) => (),
                Item::TraitAlias(_) => (),
                Item::Type(_) => (),
                Item::Union(_) => (),
                Item::Use(_) => (),
                Item::Verbatim(_) => (),
                _ => (),
            }
        }
        let new_src = quote! { #ast }.to_string();
        // println!("{}", new_src);
    }
}
