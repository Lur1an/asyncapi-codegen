use std::{env, path::Path, process::Command};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("asyncapi.rs");
    let spec_path = Path::new("./asyncapi.yaml");
    let codegen = modelina_fix::generate_models_from_spec(spec_path);
    std::fs::write(&dest_path, codegen).unwrap();
    Command::new("rustfmt")
        .arg(&dest_path)
        .output()
        .expect("Failed to format generated code");
}
