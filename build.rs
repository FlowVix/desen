use wgsl_grease::{Error, wgpu_types};

fn main() -> Result<(), Error> {
    println!("cargo::rerun-if-changed=src/shaders/main.wgsl");

    if !std::fs::exists("src/shaders/out").unwrap() {
        std::fs::create_dir("src/shaders/out").unwrap();
    }

    wgsl_grease::WgslBindgenBuilder::default()
        .shader_root("src/shaders")
        .add_shader("main.wgsl")
        .output("src/shaders/out")
        .separate_files(true)
        .build()
        .inspect_err(|e| eprintln!("{e:#?}"))?
        .generate()?;

    Ok(())
}
