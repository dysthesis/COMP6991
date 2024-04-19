use crate::command::command_variable_finder;
use crate::graph::update_dependencies;
use crate::spreadsheet::Spreadsheet;
use log::info;
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::CommandRunner;
use rsheet_lib::connect::Manager;
use rsheet_lib::connect::Reader;
use rsheet_lib::connect::Writer;
use rsheet_lib::replies::Reply;
use std::error::Error;

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
            Some(verb) => match *verb {
                "get" => {
                    let cell: &str = match commands.get(1) {
                        Some(val) => *val,
                        None => {
                            send.write_message(Reply::Error(format!(
                                "Insufficient arguments for 'get' command. Expected a cell number."
                            )))?;
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
                            send.write_message(Reply::Error(format!(
                                "Insufficient arguments for 'set' command. Expected a cell number."
                            )))?;
                            continue;
                        }
                    };

                    if commands.len() < 3 {
                        send.write_message(Reply::Error(format!("Insufficient command length. Expected an expression to set the value of cell {cell} to.")))?;
                        continue;
                    };
                    spreadsheet
                        .commands
                        .insert(cell.into(), commands[2..].join(" "));

                    let command: CommandRunner = match spreadsheet.commands.get(cell.into()) {
                        Some(val) => CommandRunner::new(val.as_str()),
                        None => {
                            send.write_message(Reply::Error(format!("Could not find the command even though we have only inserted it immediately before this.")))?;
                            continue;
                        }
                    };

                    let variables = command_variable_finder(&command, &spreadsheet)?;

                    spreadsheet
                        .values
                        .insert(cell.into(), command.run(&variables));

                    // TODO: Make this async
                    // Iterate through the commands vector and update the values for any of them that depends on this cell
                    let (to_update, errors) = update_dependencies(cell, &spreadsheet);
                    if errors.is_empty() {
                        spreadsheet.values.extend(to_update);
                    } else {
                        errors.iter().for_each(|err| {
                            // TODO: Figure out what to do with the errors here
                            let _ = send.write_message(Reply::Error(err.into()));
                        });
                        continue;
                    }
                    Ok(())
                }
                _ => {
                    send.write_message(Reply::Error(format!("Unrecognised command.")))?;
                    continue;
                }
            },
            None => todo!("make this error out"),
        };
    }
}
