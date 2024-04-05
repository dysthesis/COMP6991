use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::{CellArgument, CommandRunner};
use rsheet_lib::connect::{Manager, Reader, Writer};
use rsheet_lib::replies::Reply;

use std::collections::HashMap;
use std::error::Error;

use log::info;

/// A struct which encapsulates the Spreadsheet itself.
struct Spreadsheet {
    /// A hashmap which stores all of the values in the spreadsheet.
    /// Consists of a key, a String, which represents the cell number,
    /// and a value, the corresponding CellValue
    values: HashMap<String, CellValue>,

    /// A hashmap which maps a cell number to its corresponding cell command.
    commands: HashMap<String, CommandRunner>,
}

impl Spreadsheet {
    fn new() -> Self {
        Spreadsheet {
            values: HashMap::new(),
            commands: HashMap::new(),
        }
    }
}

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let (mut recv, mut send) = manager.accept_new_connection().unwrap();
    let mut spreadsheet: Spreadsheet = Spreadsheet::new();
    loop {
        info!("Just got message");
        let msg: String = recv.read_message()?;
        let commands: Vec<&str> = msg.split_whitespace().collect::<Vec<&str>>();

        let _result = match commands.first() {
            Some(verb) => {
                match *verb {
                    "get" => {
                        let cell: &str = match commands.get(1) {
                            Some(val) => *val,
                            None => {
                                send.write_message(Reply::Error(format!("Insufficient arguments for 'get' command. Expected a cell number.")))?;
                                continue;
                            }
                        };

                        let value: CellValue = match spreadsheet.values.get(cell) {
                            Some(val) => val.clone(),
                            None => {
                                send.write_message(Reply::Error(format!(
                                    "Could not find a value for the cell {cell}"
                                )))?;
                                continue;
                            }
                        };
                        send.write_message(Reply::Value(cell.to_string(), value))
                    }
                    "set" => {
                        let cell: &str = match commands.get(1) {
                            Some(val) => *val,
                            None => {
                                send.write_message(Reply::Error(format!("Insufficient arguments for 'set' command. Expected a cell number.")))?;
                                continue;
                            }
                        };

                        if commands.len() < 3 {
                            send.write_message(Reply::Error(format!("Insufficient command length. Expected an expression to set the value of cell {cell} to.")))?;
                            continue;
                        };
                        spreadsheet.commands.insert(
                            cell.into(),
                            CommandRunner::new(commands[2..].join(" ").as_str()),
                        );

                        let command = CommandRunner::new(commands[2..].join(" ").as_str());
                        let referenced: Vec<String> = command.find_variables();
                        let variables: HashMap<String, CellArgument> =
                            referenced.iter().fold(HashMap::new(), |mut acc, key| {
                                if let Some(&ref value) = spreadsheet.values.get(key) {
                                    acc.insert(key.into(), CellArgument::Value(value.clone()));
                                }
                                acc
                            });
                        spreadsheet
                            .values
                            .insert(cell.into(), command.run(&variables));
                        Ok(())
                    }
                    _ => {
                        send.write_message(Reply::Error(format!("Unrecognised command.")))?;
                        continue;
                    }
                }
            }
            None => todo!("make this error out"),
        };
    }
}
