use rsheet_lib::cell_value::CellValue;
use rsheet_lib::cells::{column_name_to_number, column_number_to_name};
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
    commands: HashMap<String, String>,
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

fn command_variable_finder(
    command: &CommandRunner,
    spreadsheet: &Spreadsheet,
) -> Result<HashMap<String, CellArgument>, &'static str> {
    let referenced: Vec<String> = command.find_variables();

    referenced.iter().try_fold(HashMap::new(), |mut acc, key| {
                            if key.contains("_") {
                                let cells = list_cells_in_range(key)?;
                                let values_matrix = cells_to_value(cells, &spreadsheet.values)?;
                                if values_matrix.len() == 1 {
                                    acc.insert(
                                        key.clone(),
                                        CellArgument::Vector(values_matrix.first().expect("If we have made it this far, the matrix should have at least one vector.").to_owned()),
                                    );
                                } else {
                                    acc.insert(key.clone(), CellArgument::Matrix(values_matrix));
                                }
                            } else {
                                if let Some(value) = spreadsheet.values.get(key) {
                                    acc.insert(key.clone(), CellArgument::Value(value.clone()));
                                } else {
                                    return Err("Missing key for individual cell reference");
                                }
                            }
                            Ok(acc)
                        })
}

fn update_dependencies(
    cell: &str,
    spreadsheet: &Spreadsheet,
) -> (HashMap<String, CellValue>, Vec<String>) {
    dbg!(format!("Updating dependents of cell {cell}"));
    let mut res: (HashMap<String, CellValue>, Vec<String>) = spreadsheet
        .commands
        .iter()
        .filter(|(_key, val)| val.contains(cell))
        .fold((HashMap::new(), Vec::new()), |mut acc, (key, val)| {
            let command = CommandRunner::new(val);
            match command_variable_finder(&command, &spreadsheet) {
                Ok(val) => {
                    acc.0
                        .insert(dbg!(key.into()), dbg!(command.run(&dbg!(val))));
                }
                Err(e) => {
                    acc.1.push(format!("{e}: {key}"));
                }
            };
            acc
        });

    let (dependents, mut errors): (HashMap<String, CellValue>, Vec<String>) =
        res.0
            .iter()
            .fold((HashMap::new(), Vec::new()), |mut acc, (key, _)| {
                let mut extras = update_dependencies(&*key, spreadsheet);
                acc.0.extend(extras.0);
                acc.1.append(&mut extras.1);
                acc
            });

    res.0.extend(dependents);
    res.1.append(&mut errors);
    res
}

fn parse_cell(cell: &str) -> Result<(String, u32), &'static str> {
    let col_part = cell
        .chars()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();
    let row_part: String = cell.chars().filter(|c| c.is_numeric()).collect();

    if col_part.is_empty() || row_part.is_empty() {
        return Err("Invalid cell format");
    }

    let row_num: u32 = row_part.parse().map_err(|_| "Invalid number in cell")?;

    Ok((col_part, row_num))
}
fn cells_to_value(
    cell_names: Vec<Vec<String>>,
    data_map: &HashMap<String, CellValue>,
) -> Result<Vec<Vec<CellValue>>, &'static str> {
    cell_names
        .into_iter()
        .map(|col| {
            col.into_iter()
                .map(|cell_name| {
                    data_map
                        .get(&cell_name)
                        .cloned()
                        .ok_or("Missing key in HashMap") // Consider borrowing if CellValue is large and if lifetime semantics allow
                })
                .collect::<Result<Vec<_>, _>>() // Collect inner vector results, short-circuiting on error
        })
        .collect() // Collect outer vector results, short-circuiting on error
}
pub fn list_cells_in_range(range: &str) -> Result<Vec<Vec<String>>, &'static str> {
    let parts: Vec<&str> = range.split('_').collect();
    if parts.len() != 2 {
        return Err("Range must be in 'start_end' format");
    }

    let (start_col, start_row) = parse_cell(parts[0])?;
    let (end_col, end_row) = parse_cell(parts[1])?;

    let col_start = column_name_to_number(&start_col);
    let col_end = column_name_to_number(&end_col);

    let columns = (col_start..=col_end)
        .map(|col_num| {
            let col_name = column_number_to_name(col_num);
            (start_row..=end_row)
                .map(move |row_num| format!("{}{}", col_name, row_num))
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();

    Ok(columns)
}
