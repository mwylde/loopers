use std::path::PathBuf;
use std::{env, fs};

fn main() {
    // copy audio files
    let mut out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_dir.push("../resources");
    if let Err(err) = fs::create_dir(&out_dir) {
        if err.kind() != std::io::ErrorKind::AlreadyExists {
            panic!("Failed to create resource directory: {}", err);
        }
    }

    let mut resources = env::current_dir().unwrap();
    resources.push("../resources");

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
