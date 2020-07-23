#[macro_use]
extern crate log;

pub mod config;
pub mod error;
pub mod music;
pub mod protos;
pub mod gui_channel;

pub fn clamp<T: PartialOrd + Copy>(v: T, min: T, max: T) -> T {
    assert!(min <= max);
    let mut x = v;
    if x < min {
        x = min;
    }
    if x > max {
        x = max;
    }
    x
}

pub fn f32_to_i16(v: f32) -> i16 {
    let v = clamp(v, -1.0, 1.0);
    (v * 32768.0).floor() as i16
}