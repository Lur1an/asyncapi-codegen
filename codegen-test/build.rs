use std::{env, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("asyncapi.rs");
    let spec_path = Path::new("./asyncapi.yaml");
    let codegen = modelina_rs::generate_models_from_spec(spec_path);
    std::fs::write(&dest_path, codegen).unwrap();
}
