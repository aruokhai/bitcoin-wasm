use std::{env, fs::{read_to_string, write}, path::{Path, PathBuf}, process::Command};
use wit_component::ComponentEncoder;


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
        .get("runnercomponent")
        .unwrap()
        .as_object(). unwrap();
        
    
    for (key, path) in targets.into_iter() {
        
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
        let mut cmd = Command::new("cargo-component");
        cmd.arg("build")
            .arg(format!("--package={}",&key))
            .env("CARGO_TARGET_DIR", &out_dir)
            .env("CARGO_PROFILE_DEV_DEBUG", "1");
            println!("running: {cmd:?}");
            let status = cmd.status().unwrap();
            assert!(status.success());

        let mut wit_world = Vec::new();
        wit_world.push("wasmtime::component::bindgen!({\n".to_string());
        wit_world.push("inline: \"".to_string());
        let wit_path = path.as_object().unwrap().get("path").unwrap().as_str().unwrap();
        println!("hello wit{}",wit_path);

        for line in read_to_string(wit_path).unwrap().lines() {
            if line.contains("import") {
                continue;
            }
            if line.contains("///") {
                continue;
            }
            wit_world.push(line.to_string());
            wit_world.push("\n".to_string());
        }

        wit_world.push("\"});".to_string());
        let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join(format!("{}_WIT.rs", &key));
        write(out_dir, wit_world.join("")).unwrap();
    }
   
    println!("done with generating build details");
       
}


