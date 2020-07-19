extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["protos/loopers.proto"], &["protos/"]).unwrap();
}
