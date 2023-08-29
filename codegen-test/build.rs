use std::{env, fs, path::Path, process::Command};
fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("asyncapi.rs");
    let spec_path = Path::new("./asyncapi.yaml");
    let yaml_data = fs::read_to_string(spec_path).unwrap();
    let codegen = asyncapi_models::generate_code_from_yaml(&yaml_data);
    std::fs::write(&dest_path, codegen).unwrap();
    Command::new("rustfmt")
        .arg(&dest_path)
        .output()
        .expect("Failed to format generated code");
}
