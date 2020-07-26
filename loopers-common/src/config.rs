use serde::{Deserialize, Serialize};
use std::io;
use std::str::FromStr;
use crate::api::Command;
use std::fs::File;
use csv::StringRecord;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Config {
    pub midi_mappings: Vec<MidiMapping>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MidiMapping {
    pub channel: u8,
    pub data: u8,
    pub command: Command,
}

impl MidiMapping {
    pub fn from_file(name: &str, file: &File) -> io::Result<Vec<MidiMapping>> {
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(b'\t')
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
                        error!("Failed to load midi mapping on line {}: {}", pos.line(), err);
                    } else {
                        error!("Failed to load midi mapping: {}", err);
                    }
                },
            }
        }

        if caught_error {
            Err(io::Error::new(io::ErrorKind::Other,
                               format!("Failed to parse midi mappings from {}", name)))
        } else {
            Ok(mappings)
        }
    }

    fn from_record(record: &StringRecord) -> Result<MidiMapping, String> {
        let channel = record.get(0).ok_or("No channel field".to_string())
            .and_then(|c| u8::from_str(c).map_err(|_| "Channel is not a number".to_string()))?;

        let data = record.get(1).ok_or("No data field".to_string())
            .and_then(|c| u8::from_str(c).map_err(|_| "Data is not a number".to_string()))?;

        let args: Vec<&str> = record.iter().skip(3).collect();

        let command = record.get(2).ok_or("No command field".to_string())
            .and_then(|c| Command::from_str(c, &args))?;

        Ok(MidiMapping {
            channel,
            data,
            command
        })
    }
}
