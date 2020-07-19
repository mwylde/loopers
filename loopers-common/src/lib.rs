use crate::music::{FrameTime, MetricStructure};

pub mod config;
pub mod error;
pub mod music;
pub mod protos;

#[derive(Copy, Clone, Debug)]
pub struct EngineStateSnapshot {
    pub time: FrameTime,
    pub metric_structure: MetricStructure,
    pub active_looper: u32,
    pub looper_count: usize,
}

#[derive(Copy, Clone, Debug)]
pub enum GuiCommand {
    StateSnapshot(EngineStateSnapshot),
    AddLooper(u32),
    RemoveLooper(u32),
}