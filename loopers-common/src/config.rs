use crate::api::Command;
use csv::StringRecord;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::str::FromStr;

#[cfg(test)]
mod tests {
    use crate::api::{Command, LooperCommand, LooperTarget};
    use crate::config::MidiMapping;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_midi_mapping() {
        let mut file = NamedTempFile::new().unwrap();
        {
            let file = file.as_file_mut();
            writeln!(file, "*\t22\t127\tRecordOverdubPlay\t0").unwrap();
            writeln!(file, "*\t1\t23\t5\tMute\t3").unwrap();
            writeln!(file, "1\t24\t6\tStart").unwrap();
            writeln!(file, "1\t24\t10-50\tSetPan").unwrap();
            file.flush().unwrap();
        }

        let mapping = MidiMapping::from_file(
            &file.path().to_string_lossy(),
            &File::open(&file.path()).unwrap(),
        );

        assert_eq!(
            vec![
                MidiMapping {
                    channel: 22,
                    data: 127,
                    command: Command::Looper(
                        LooperCommand::RecordOverdubPlay,
                        LooperTarget::Index(0)
                    ),
                },
                MidiMapping {
                    channel: 23,
                    data: 5,
                    command: Command::Looper(LooperCommand::Mute, LooperTarget::Index(3)),
                },
                MidiMapping {
                    channel: 24,
                    data: 6,
                    command: Command::Start,
                },
            ],
            mapping.unwrap()
        );
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

pub enum DataValue {
    Any,
    Range(u8, u8),
    Value(u8),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct MidiMapping {
    pub channel: Option<u8>,
    pub controller: u8,
    pub data: DataValue,
    pub command: Command,
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
            .ok_or("No channel field".to_string())
            .and_then(|c| u8::from_str(c).map_err(|_| "Channel is not a number".to_string()))?;

        let data = record
            .get(1)
            .ok_or("No data field".to_string())
            .and_then(|c| u8::from_str(c).map_err(|_| "Data is not a number".to_string()))?;

        let args: Vec<&str> = record.iter().skip(3).collect();

        let command = record
            .get(2)
            .ok_or("No command field".to_string())
            .and_then(|c| Command::from_str(c, &args))?;

        Ok(MidiMapping {
            channel,
            data,
            command,
        })
    }
}
