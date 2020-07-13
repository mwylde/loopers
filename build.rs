extern crate prost_build;
extern crate serde;
extern crate tower_grpc_build;

use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let mut path = env::current_dir().unwrap();
    path.push("src");

    let mut prost_config = prost_build::Config::new();
    prost_config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    //prost_config.out_dir(path);

    tower_grpc_build::Config::from_prost(prost_config)
        .enable_server(true)
        .enable_client(true)
        .build(&["protos/loopers.proto"], &["protos"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));

    // copy audio files
    let mut out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_dir.push("resources");
    if let Err(err) = fs::create_dir(&out_dir) {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Failed to create resource directory: {}", err);
        }
    }

    let mut resources = env::current_dir().unwrap();
    resources.push("resources");

    fs::copy(
        resources.join("sine_normal.wav"),
        &out_dir.join("sine_normal.wav"),
    )
    .unwrap();
    fs::copy(
        resources.join("sine_emphasis.wav"),
        &out_dir.join("sine_emphasis.wav"),
    )
    .unwrap();
}
