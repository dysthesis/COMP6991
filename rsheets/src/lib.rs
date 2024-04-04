use rsheet_lib::cell_value::CellValue;
use rsheet_lib::command_runner::CommandRunner;
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
    /// Unlike with `values`, not all cells with existing values will have a corresponding
    /// command.
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
    let spreadsheet: Spreadsheet = Spreadsheet::new();
    loop {
        info!("Just got message");
        let msg: String = recv.read_message()?;
        send.write_message(Reply::Error(format!("{msg:?}")))?;
    }
}
