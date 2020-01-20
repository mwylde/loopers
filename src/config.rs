use crate::protos::{LooperCommandType, GlobalCommandType, LooperCommand};
use serde::{Serialize, Deserialize};
use crate::protos;
use crate::protos::command::CommandOneof;
use std::io;
use std::str::FromStr;
use std::convert::TryFrom;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Config {
    pub midi_mappings: Vec<MidiMapping>
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct MidiMapping {
    pub controller: u8,
    pub data: u8,
    pub command: Command,
}

impl MidiMapping {
    pub fn from_line(line: &str) -> io::Result<MidiMapping> {
        let err = |err: &'static str| io::Error::new(io::ErrorKind::Other, err);

        let mut cs  = line.split_ascii_whitespace();
        let controller = cs.next().ok_or(err("No controller field"))
            .and_then(|c| u8::from_str(c)
                .map_err(|e| err("Channel is not a number")))?;

        let data = cs.next().ok_or(err("No data field"))
            .and_then(|c| u8::from_str(c)
                .map_err(|e| err("Data is not a number")))?;

        let command_name = cs.next().ok_or(err("No command field"))?;

        let target = cs.next();

        let mut buf = String::new();
        buf.push_str(&format!("controller: {}\n", controller));
        buf.push_str(&format!("data: {}\n", data));
        buf.push_str("command:\n");
        buf.push_str(&format!("  command: {}\n", command_name));
        if let Some(target) = target {
            if let Ok(i) = u32::from_str(target) {
                buf.push_str(&format!("  target:\n    Number: {}\n", i))
            } else {
                buf.push_str(&format!("  target: {}", target));
            }
        }

        // println!("MAPPING\n-----------------\n{}", buf);

        serde_yaml::from_str(&buf).map_err(|e| {
            io::Error::new(io::ErrorKind::Other, e)
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
// #[serde(untagged)]
pub enum LooperCommandTarget {
    All,
    Selected,
    Number(u32),
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[serde(untagged)]
pub enum Command {
    LooperCommand {
        command: LooperCommandType,
        target: LooperCommandTarget
    },
    GlobalCommand {
        command: GlobalCommandType
    }
}

impl Config {
    pub fn to_config(&self) -> protos::Config {
        let midi_mappings: Vec<protos::MidiMapping> = self.midi_mappings.iter().map(|m| {
            let command = match &m.command {
                Command::LooperCommand { command, target } => {
                    protos::Command {
                        command_oneof: Some(protos::command::CommandOneof::LooperCommand(
                            protos::LooperCommand {
                                command_type: *command as i32,
                                target_oneof: Some(match target {
                                    LooperCommandTarget::All =>
                                        protos::looper_command::TargetOneof::TargetAll(
                                            protos::TargetAll{}),
                                    LooperCommandTarget::Selected =>
                                        protos::looper_command::TargetOneof::TargetSelected(
                                            protos::TargetSelected{}),
                                    LooperCommandTarget::Number(i) =>
                                    protos::looper_command::TargetOneof::TargetNumber(
                                        protos::TargetNumber {
                                            looper_number: *i
                                        }
                                    ),
                                })
                            }
                        ))
                    }
                },
                Command::GlobalCommand { command } => {
                    protos::Command {
                        command_oneof: Some(protos::command::CommandOneof::GlobalCommand(
                            protos::GlobalCommand {
                                command: *command as i32
                            }
                        )),
                    }
                },
            };
            return protos::MidiMapping {
                controller_number: m.controller as u32,
                data: m.data as u32,
                command: Some(command),
            };
        }).collect();

        return protos::Config {
            midi_mappings
        };
    }
}
