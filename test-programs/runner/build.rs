use std::{env, fs, path::{Path, PathBuf}, process::Command};
use wit_component::ComponentEncoder;

// #[derive(Debug, Deserialize)]
// struct Dependencies {
//     dep: Vec<Dependency>,
    
// }

// #[derive(Debug, Deserialize)]
// struct Dependency {
//     name: String,
//     deps: Vec<Dependency>
// }

fn main() {
    println!("cargo:rerun-if-changed=../../");
    build_and_generate_tests();
}

fn build_and_generate_tests() {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    println!("running: runner");
    compose_dep(out_dir);


}

fn build_dep(out_dir: PathBuf){
    let mut cmd = Command::new("cargo-component");
    cmd.arg("build")
    .arg("--package=artifacts")
    .env("CARGO_TARGET_DIR", &out_dir)
    .env("CARGO_PROFILE_DEV_DEBUG", "1");
    println!("running: {cmd:?}");
    let status = cmd.status().unwrap();
    assert!(status.success());
    let mut cmd = Command::new("cargo-component");
    cmd.arg("build")
        .arg("--package=store")
        .env("CARGO_TARGET_DIR", &out_dir)
        .env("CARGO_PROFILE_DEV_DEBUG", "1");
        
    eprintln!("running: {cmd:?}");
    let status = cmd.status().unwrap();
    assert!(status.success());
    
}


fn compose_dep(out_dir: PathBuf) {
    build_dep(out_dir.clone());
    let mut wac = Command::new("wac");
    let binding = out_dir.clone()
    .join("wasm32-wasi")
    .join("debug")
    .join(format!("store.wasm"));
    let store_path = binding.to_str().unwrap();
    println!("store path {store_path:?}");
    let binding = out_dir
        .join("wasm32-wasi")
        .join("debug")
        .join(format!("artifacts.wasm"));
    let artifact_path = binding.to_str().unwrap();
    let binding = out_dir
        .join(format!("test.wasm"));
    let output_path = binding.to_str().unwrap();
    wac.arg("plug")
        .arg(format!("{artifact_path}"))
        .arg("--plug")
        .arg(format!("{store_path}"))
        .arg("-o")
        .arg(format!("{output_path}"));
    let status = wac.status().unwrap();
        assert!(status.success());
}

// fn parse_yaml() {
//     let f = std::fs::File::open("dependencies.yaml")?;
//     let d: Dependencies = serde_yaml::from_reader(f)?;

//     for dependency in d.dep.iter() {

//     }
// }

// fn compile_dep(dep: &Dependency, mut checked_list: Vec<string> ) {
//     if dep.deps.len() == 0 {
//         let package_name = dep.name;
//         cmd.arg("component")
//         .arg("build")
//         .arg(format!("--package={package_name}"));
//         let status = cmd.status().unwrap();
//         assert!(status.success());
//         checked_list.push(dep.name.clone());
//         return;
//     } else {
//         for  depend in dep.deps.iter() {
//             if checked_list.binary_search(depend).is_err(){
//                 compile_dep(depend, checked_list)
//             }

//         }
        
//     }


//     return;
    

// }

// Compile a component, return the path of the binary:
// fn compile_components(wasm: &Path, adapter: &[u8]) -> PathBuf {
//     println!("creating a component from {wasm:?}");
//     let module = fs::read(wasm).expect("read wasm module");
//     let component = ComponentEncoder::default()
//         .module(module.as_slice())
//         .unwrap()
//         .
//         .validate(true)
//         .encode()
//         .expect("module can be translated to a component");
//     let out_dir = wasm.parent().unwrap();
//     let stem = wasm.file_stem().unwrap().to_str().unwrap();
//     let component_path = out_dir.join(format!("{stem}.component.wasm"));
//     fs::write(&component_path, component).expect("write component to disk");
//     component_path
// }


fn cargo() -> Command {
    // Miri configures its own sysroot which we don't want to use, so remove
    // miri's own wrappers around rustc to ensure that we're using the real
    // rustc to build these programs.
    let mut cargo = Command::new("cargo-component");
    if std::env::var("CARGO_CFG_MIRI").is_ok() {
        cargo.env_remove("RUSTC").env_remove("RUSTC_WRAPPER");
    }
    cargo
}