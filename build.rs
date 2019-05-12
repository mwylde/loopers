extern crate tower_grpc_build;
extern crate prost_build;

use std::env;

fn main() {
    let mut path = env::current_dir().unwrap();
    path.push("src");

    let mut prost_config = prost_build::Config::new();
    prost_config.out_dir(path);

    tower_grpc_build::Config::from_prost(prost_config)
        .enable_server(true)
        .enable_client(true)
        .build(
            &["protos/loopers.proto"],
            &["protos"],
        )
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
