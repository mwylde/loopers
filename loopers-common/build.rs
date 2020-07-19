extern crate prost_build;
extern crate serde;
extern crate tower_grpc_build;

fn main() {
    let mut prost_config = prost_build::Config::new();
    prost_config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    tower_grpc_build::Config::from_prost(prost_config)
        .enable_server(true)
        .enable_client(true)
        .build(&["protos/loopers.proto"], &["protos"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
