#[macro_use]
extern crate log;
extern crate derive_more;

use crate::api::FrameTime;
use crate::music::MetricStructure;

pub mod api;
pub mod config;
pub mod gui_channel;
pub mod midi;
pub mod music;

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

#[allow(unused_variables)]
pub trait Host<'a> {
    fn add_looper(&mut self, id: u32) -> Result<(), String>;
    fn remove_looper(&mut self, id: u32) -> Result<(), String>;

    fn output_for_looper<'b>(&'b mut self, id: u32) -> Option<[&'b mut [f32]; 2]>
    where
        'a: 'b;

    fn start_transport(&mut self) {}
    fn stop_transport(&mut self) {}
    fn set_transport_position(&mut self, time: FrameTime, metric_structure: MetricStructure) {}
    fn set_transport_bpm(&mut self, bpm: f32) {}
}
