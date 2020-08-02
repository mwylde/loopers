use loopers_common::api::{Command, FrameTime};
use loopers_common::music::MetricStructure;
use std::cmp::Ordering;

#[cfg(test)]
mod tests {
    use crate::trigger::{Trigger, TriggerCondition};
    use loopers_common::api::{Command, FrameTime};
    use loopers_common::music::{MetricStructure, Tempo, TimeSignature};
    use proptest::prelude::*;

    fn correct_measure_trigger(trigger: &Trigger) -> FrameTime {
        let mut t = trigger.start_time;
        loop {
            if t.0 >= 0
                && t.0 % trigger.metric_structure.tempo.samples_per_beat() as i64 == 0
                && trigger
                    .metric_structure
                    .time_signature
                    .beat_of_measure(trigger.metric_structure.tempo.beat(t))
                    == 0
            {
                return FrameTime(t.0);
            }
            t = FrameTime(t.0 + 1);
        }
    }

    #[test]
    fn test_measure_trigger() {
        let ms = MetricStructure {
            tempo: Tempo::from_bpm(120.0),
            time_signature: TimeSignature::new(4, 4).unwrap(),
        };

        let t = Trigger::new(TriggerCondition::Measure, Command::Start, ms, FrameTime(0)).unwrap();

        assert_eq!(FrameTime(0), t.triggered_at());

        let t = Trigger::new(TriggerCondition::Measure, Command::Start, ms, FrameTime(1)).unwrap();

        assert_eq!(FrameTime(88200), t.triggered_at());

        let t = Trigger::new(
            TriggerCondition::Measure,
            Command::Start,
            ms,
            FrameTime(-30000),
        )
        .unwrap();

        assert_eq!(FrameTime(0), t.triggered_at());

        let t = Trigger::new(
            TriggerCondition::Measure,
            Command::Start,
            ms,
            FrameTime(88200),
        )
        .unwrap();

        assert_eq!(FrameTime(88200), t.triggered_at());
    }

    proptest! {
        #[test]
        fn test_measure_trigger_prop(tempo in 1f32..220.0, lower in 2u8..32, upper in 1u8..7, time in -10i64..100_000_000) {
            let ms = MetricStructure {
                tempo: Tempo::from_bpm(tempo),
                time_signature: TimeSignature::new(lower, 2u8.pow(upper as u32)).unwrap(),
            };

            let t = Trigger::new(TriggerCondition::Measure,
                                 Command::Start, ms, FrameTime(time)).unwrap();


            assert_eq!(correct_measure_trigger(&t), t.triggered_at());
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(dead_code)]
pub enum TriggerCondition {
    Measure,
    Beat(u8),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Trigger {
    pub condition: TriggerCondition,
    pub command: Command,
    metric_structure: MetricStructure,
    start_time: FrameTime,
    triggered_at: FrameTime,
}

impl Trigger {
    pub fn new(
        condition: TriggerCondition,
        command: Command,
        metric_structure: MetricStructure,
        start_time: FrameTime,
    ) -> Option<Trigger> {
        let valid = match condition {
            TriggerCondition::Measure => true,
            TriggerCondition::Beat(b) => b < metric_structure.time_signature.upper,
        };

        if valid {
            let triggered_at = Self::compute_triggered_at(condition, metric_structure, start_time);
            Some(Trigger {
                condition,
                command,
                metric_structure,
                start_time,
                triggered_at,
            })
        } else {
            None
        }
    }

    fn compute_triggered_at(
        condition: TriggerCondition,
        metric_structure: MetricStructure,
        start_time: FrameTime,
    ) -> FrameTime {
        match condition {
            TriggerCondition::Measure => {
                if start_time.0 < 0 {
                    FrameTime(0)
                } else {
                    let spb = metric_structure.tempo.samples_per_beat() as i64;
                    let samples_per_measure = spb * metric_structure.time_signature.upper as i64;
                    let rem = start_time.0 % samples_per_measure;

                    if rem == 0 {
                        FrameTime(start_time.0)
                    } else {
                        FrameTime(start_time.0 + (samples_per_measure - rem))
                    }
                }
            }
            TriggerCondition::Beat(_) => unimplemented!(),
        }
    }

    pub fn triggered_at(&self) -> FrameTime {
        self.triggered_at
    }
}

impl Eq for Trigger {}

impl PartialOrd for Trigger {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.triggered_at.partial_cmp(&other.triggered_at)
    }
}

impl Ord for Trigger {
    fn cmp(&self, other: &Self) -> Ordering {
        self.triggered_at.cmp(&other.triggered_at)
    }
}
