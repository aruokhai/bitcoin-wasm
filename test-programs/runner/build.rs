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
    compose_test_component();
    // build_and_generate_tests();
}


fn compose_test_component() {
    let meta = cargo_metadata::MetadataCommand::new().exec().unwrap();
    let targets = meta
        .packages
        .iter()
        .find(|p| p.name == "runner")
        .unwrap()
        .metadata
        .as_object()
        .unwrap()
        .get("component")
        .unwrap()
        .as_object(). unwrap();
        

    for (key, path) in targets.into_iter() {
        let real_pathbuf = build_compose_component(&key);
        println!("Did it {:?}", real_pathbuf)

    }

    println!("{:?}", targets);
       
}

fn build_compose_component(package_name: &str) -> PathBuf {
    println!("package name is {}", package_name);
    let meta = cargo_metadata::MetadataCommand::new().exec().unwrap();
    let targets = meta
            .packages
            .iter()
            .find(|p| p.name == package_name)
            .unwrap()
            .metadata
            .as_object()
            .unwrap();
    println!("target  is {:?}", targets.clone());
    let real_targets = targets.get("component")
        .unwrap()
        .as_object()
        .unwrap()
        .get("target")
        .unwrap()
        .as_object()
        .unwrap()
        .get("dependencies").unwrap()
        .as_object().unwrap();

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let mut cmd = Command::new("cargo-component");
    cmd.arg("build")
        .arg(format!("--package={}",package_name))
        .env("CARGO_TARGET_DIR", &out_dir)
        .env("CARGO_PROFILE_DEV_DEBUG", "1");
        println!("running: {cmd:?}");
        let status = cmd.status().unwrap();
        assert!(status.success());
    let built_path = out_dir
        .join("wasm32-wasi")
        .join("debug")
        .join(format!("{}.wasm",package_name));
    
    if real_targets.is_empty() {
        return  built_path;    
    }


    let mut wac = Command::new("wac");
    let artifact_path = built_path.to_str().unwrap();
    wac.arg("plug")
    .arg(format!("{artifact_path}"));


    for (key, path) in real_targets.into_iter() {
        let modified_key = key.split(":").collect::<Vec<&str>>()[1];
        let real_path: &str = path.get("path").unwrap().as_str().unwrap();
        let path  = build_compose_component(modified_key); 
        wac.arg("--plug")
        .arg(format!("{}",path.to_str().unwrap()));
    }

    let output_path = out_dir
        .join(format!("{}{}.wasm",package_name, package_name));
    wac.arg("-o")
    .arg(format!("{}",output_path.to_str().unwrap()));
    let status = wac.status().unwrap();
    assert!(status.success());

    return  output_path;

}

