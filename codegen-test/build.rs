use std::{env, path::Path, process::Command};
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("asyncapi.rs");
    let yaml_data = include_str!("./asyncapi.yaml");
    let codegen = schema2code::generate_rust(&yaml_data);
    std::fs::write(&dest_path, codegen).unwrap();
    Command::new("rustfmt")
        .arg(&dest_path)
        .output()
        .expect("Failed to format generated code");
}
