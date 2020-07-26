extern crate prost_build;
extern crate serde;

fn main() {
    let mut prost_config = prost_build::Config::new();
    prost_config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    prost_config.compile_protos(&["protos/loopers.proto"], &["protos"])
        .unwrap_or_else(|e| panic!("protobuf compilation failed: {}", e));
}
