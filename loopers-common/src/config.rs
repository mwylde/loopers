use crate::api::{Command, CommandData};
use csv::StringRecord;
use std::fs::File;
use std::io;
use std::str::FromStr;
use crate::midi::MidiEvent;

#[cfg(test)]
mod tests {
    use crate::api::{Command, LooperCommand, LooperTarget, CommandData};
    use crate::config::{MidiMapping, DataValue};
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use crate::api::LooperCommand::RecordOverdubPlay;

    #[test]
    fn test_load_midi_mapping() {
        let mut file = NamedTempFile::new().unwrap();
        {
            let file = file.as_file_mut();
            writeln!(file, "*\t22\t127\tRecordOverdubPlay\t0").unwrap();
            writeln!(file, "*\t1\t23\t5\tMute\t3").unwrap();
            writeln!(file, "1\t24\t6\tStart").unwrap();
            writeln!(file, "1\t24\t10-50\tSetPan\tSelected\tdata").unwrap();
            file.flush().unwrap();
        }

        let mapping = MidiMapping::from_file(
            &file.path().to_string_lossy(),
            &File::open(&file.path()).unwrap(),
        ).unwrap();

        assert_eq!(None, mapping[0].channel);
        assert_eq!(22, mapping[0].controller);
        assert_eq!(DataValue::Value(127), mapping[0].data);
        assert_eq!(Command::Looper(RecordOverdubPlay, LooperTarget::Index(0)),
                   (mapping[0].command)(CommandData {data:  127 }));
        //
        // assert_eq!(
        //     vec![
        //         MidiMapping {
        //             channel: None,
        //             controller: 22,
        //             data: DataValue::Value(127),
        //             command: Command::Looper(
        //                 LooperCommand::RecordOverdubPlay,
        //                 LooperTarget::Index(0)
        //             ),
        //         },
        //         MidiMapping {
        //             channel: 23,
        //             data: 5,
        //             command: Command::Looper(LooperCommand::Mute, LooperTarget::Index(3)),
        //         },
        //         MidiMapping {
        //             channel: 24,
        //             data: 6,
        //             command: Command::Start,
        //         },
        //     ],
        //     mapping.unwrap()
        // );
    }
}

pub struct Config {
    pub midi_mappings: Vec<MidiMapping>,
}

impl Config {
    pub fn new() -> Config {
        Config {
            midi_mappings: vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum DataValue {
    Any,
    Range(u8, u8),
    Value(u8),
}

impl DataValue {
    fn parse(s: &str) -> Option<DataValue> {
        if s == "*" {
            return Some(DataValue::Any);
        }

        if let Ok(v) = u8::from_str(s) {
            if v <= 127 {
                return Some(DataValue::Value(v));
            }
        }

        let split: Vec<u8> = s.split("-")
            .filter_map(|s| u8::from_str(s).ok())
            .collect();

        if split.len() == 2 && split[0] < 127 && split[1] < 127 && split[0] < split[1] {
            return Some(DataValue::Range(split[0], split[1]))
        }

        None
    }

    fn matches(&self, data: u8) -> bool {
        match self {
            DataValue::Any => true,
            DataValue::Range(a, b) => (*a..=*b).contains(&data),
            DataValue::Value(a) => *a == data,
        }
    }
}

pub struct MidiMapping {
    pub channel: Option<u8>,
    pub controller: u8,
    pub data: DataValue,
    pub command: Box<dyn Fn(CommandData) -> Command + Send>,
}

impl MidiMapping {
    pub fn from_file(name: &str, file: &File) -> io::Result<Vec<MidiMapping>> {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
            .flexible(true)
            .has_headers(false)
            .from_reader(file);

        let mut mappings = vec![];
        let mut caught_error = false;

        for result in rdr.records() {
            let record = result?;

            match Self::from_record(&record) {
                Ok(mm) => mappings.push(mm),
                Err(err) => {
                    caught_error = true;
                    if let Some(pos) = record.position() {
                        error!(
                            "Failed to load midi mapping on line {}: {}",
                            pos.line(),
                            err
                        );
                    } else {
                        error!("Failed to load midi mapping: {}", err);
                    }
                }
            }
        }

        if caught_error {
            Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to parse midi mappings from {}", name),
            ))
        } else {
            Ok(mappings)
        }
    }

    fn from_record(record: &StringRecord) -> Result<MidiMapping, String> {
        let channel = record
            .get(0)
            .ok_or("No channel field".to_string())?;

        let channel = match channel {
            "*" => None,
            c => Some(u8::from_str(c)
                .map_err(|_| "Channel must be * or a number".to_string())
                .and_then(|c| if c >= 1 && c <= 16 { Ok(c) } else {
                    Err("Channel must be between 1 and 16".to_string())
                })?)
        };


        let controller = record
            .get(1)
            .ok_or("No controller field".to_string())
            .and_then(|c| u8::from_str(c).map_err(|_| "Controller is not a number".to_string()))?;

        let data = record
            .get(2)
            .ok_or("No data field".to_string())
            .map(DataValue::parse)?
            .ok_or("Invalid data format (expected either *, a range like 15-20, or a single value like 127")?;


        let args: Vec<&str> = record.iter().skip(3).collect();

        let command = record
            .get(2)
            .ok_or("No command field".to_string())
            .and_then(|c| Command::from_str(c, &args))?;

        Ok(MidiMapping {
            channel,
            controller,
            data,
            command,
        })
    }

    pub fn command_for_event(&self, event: &MidiEvent) -> Option<Command> {
        match event {
            MidiEvent::ControllerChange { channel, controller, data } => {
                if (self.channel.is_none() || self.channel.unwrap() == *channel) &&
                    (self.controller == *controller) &&
                    (self.data.matches(*data)) {
                    return Some((self.command)(CommandData {
                        data: *data
                    }));
                }
            }
        }

        None
    }
}
