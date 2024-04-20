use crate::spreadsheet::Spreadsheet;
use log::info;
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
        let msg: String = match recv.read_message() {
            Ok(msg) => msg,
            Err(_) => return Ok(()),
        };
        let commands: Vec<&str> = msg.split_whitespace().collect::<Vec<&str>>();

        let _result = match commands.first() {
            Some(verb) => match *verb {
                "get" => {
                    if spreadsheet.is_self_referential() {
                        continue;
                    }
                    let cell: &str = match commands.get(1) {
                        Some(val) => *val,
                        None => {
                            send.write_message(Reply::Error(format!(
                                "Insufficient arguments for 'get' command. Expected a cell number."
                            )))?;
                            continue;
                        }
                    };
                    let reply = match spreadsheet.get(cell.to_string()) {
                        Ok(val) => Reply::Value(cell.to_string(), val),
                        Err(e) => Reply::Error(e.to_string()),
                    };
                    send.write_message(reply)
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

                    if let Err(e) = spreadsheet.set(cell.into(), commands[2..].join(" ")) {
                        send.write_message(Reply::Error(e))?;
                        continue;
                    };

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
