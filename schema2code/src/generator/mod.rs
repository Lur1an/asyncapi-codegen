mod protobuf_gen;
mod rust_gen;
use crate::parser::Entity;

pub fn generate_rust(entities: Vec<Entity>) -> String {
    rust_gen::generate_code(entities)
}

pub(crate) fn snake_case(s: &str) -> String {
    let (first, rest) = s.split_at(1);
    let first = first.chars().next().unwrap();
    let mut out = String::new();
    out.push(first.to_lowercase().next().unwrap());
    for c in rest.chars() {
        if c.is_uppercase() {
            out.push('_');
            out.push(c.to_lowercase().next().unwrap())
        } else {
            out.push(c);
        }
    }
    out
}
#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_snake_case() {
        let s = "DeezNuts";
        let snake = snake_case(s);
        assert_eq!(snake, "deez_nuts");

        let s = "deezNutsOnYourChin69420";
        let snake = snake_case(s);
        assert_eq!(snake, "deez_nuts_on_your_chin69420");
    }
}
