use crate::spreadsheet::Spreadsheet;
use crossbeam::channel::{unbounded, Receiver, Sender};
use log::info;
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::connect::{Manager, Reader, ReaderWriter, Writer};
use rsheet_lib::replies::Reply;
use std::error::Error;
use std::sync::Arc;
use std::thread;

static NUM_WORKERS: i32 = 1;

pub fn start_server<M>(mut manager: M) -> Result<(), Box<dyn Error>>
where
    M: Manager,
{
    let spreadsheet = Arc::new(Spreadsheet::new());
    let (tx, rx): (Sender<(String, String)>, Receiver<(String, String)>) = unbounded();
    thread::scope(|s| {
        s.spawn(|| spawn_workers(rx, spreadsheet.clone()));
        while let Ok((recv, send)) = manager.accept_new_connection() {
            let ss = spreadsheet.clone();
            let child_tx = tx.clone();
            s.spawn(move || handle_connection::<M>(recv, send, ss, child_tx));
            info!("Spawned new connection thread.");
        }
    });

    // If it got to this point, it probably failed to receive new connection
    Ok(())
}
fn handle_connection<M>(
    mut recv: <<M as Manager>::ReaderWriter as ReaderWriter>::Reader,
    mut send: <<M as Manager>::ReaderWriter as ReaderWriter>::Writer,
    spreadsheet: Arc<Spreadsheet>,
    sender: Sender<(String, String)>,
) -> Result<(), String>
where
    M: Manager,
{
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
                    info!("Attempting to get a cell's value");
                    let cell: &str = match commands.get(1) {
                        Some(val) => {
                            info!("Found cell name: {val}");
                            *val
                        }
                        None => {
                            info!("No cell name found");
                            // Try to fix the fact that Box<dyn Error>> isn't Send
                            let _ = send.write_message(Reply::Error(format!(
                                "Insufficient arguments for 'get' command. Expected a cell number."
                            )));
                            continue;
                        }
                    };
                    if spreadsheet.is_self_referential() {
                        let _ = send.write_message(Reply::Error(String::from(format!(
                            "The value for cell {cell} is invalid"
                        ))));

                        continue;
                    }
                    if spreadsheet.is_invalid_node(cell.to_string()) {
                        let _ = send.write_message(Reply::Error(String::from(format!(
                            "The value for cell {cell} is invalid"
                        ))));
                        continue;
                    }

                    if spreadsheet.has_invalid_dependencies(&cell.to_string()) {
                        let _ = send.write_message(Reply::Error(String::from(format!(
                            "The value for cell {cell} is invalid"
                        ))));
                        continue;
                    }
                    let val: CellValue = spreadsheet.get(cell.to_string()).unwrap_or_default();
                    info!("Value for cell {} is {:?}", cell, val);
                    send.write_message(Reply::Value(cell.to_string(), val))
                }

                "set" => {
                    info!("Attempting to set a cell's value");
                    let cell: &str = match commands.get(1) {
                        Some(val) => {
                            info!("Found cell name: {val}");
                            *val
                        }
                        None => {
                            info!("No cell name found");
                            let _ = send.write_message(Reply::Error(format!(
                                "Insufficient arguments for 'set' command. Expected a cell number."
                            )));
                            continue;
                        }
                    };

                    if commands.len() < 3 {
                        info!("No value to set the cell {cell}'s value to.");
                        let _ = send.write_message(Reply::Error(format!("Insufficient command length. Expected an expression to set the value of cell {cell} to.")));
                        continue;
                    };
                    let command = commands[2..].join(" ");
                    if let Err(e) = sender.send((cell.to_string(), command)) {
                        info!("Send error occurred: {:?}", e);
                    };

                    Ok(())
                }
                _ => {
                    let _ = send.write_message(Reply::Error(format!("Unrecognised command.")));
                    continue;
                }
            },
            None => todo!("make this error out"),
        };
    }
}

fn spawn_workers(receiver: Receiver<(String, String)>, spreadsheet: Arc<Spreadsheet>) {
    let mut children = Vec::new();
    for i in 0..NUM_WORKERS {
        info!("Spawning worker thread {i}");
        let thread_receiver = receiver.clone();
        let ss = spreadsheet.clone();
        let child = thread::spawn(move || loop {
            let (cell, command) = match thread_receiver.recv() {
                Ok(res) => dbg!(res),
                Err(_) => {
                    continue;
                }
            };
            info!("Worker thread {i} received instruction to set cell {cell} to {command}");
            let _ = ss.set(cell, command);
        });
        children.push(child);
    }
    children
        .into_iter()
        .for_each(|child| child.join().expect("child thread panicked"));
}
