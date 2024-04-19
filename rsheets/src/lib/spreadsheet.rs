use std::collections::HashMap;

use rsheet_lib::cell_value::CellValue;

/// A struct which encapsulates the Spreadsheet itself.
pub(crate) struct Spreadsheet {
    /// A hashmap which stores all of the values in the spreadsheet.
    /// Consists of a key, a String, which represents the cell number,
    /// and a value, the corresponding CellValue
    pub(crate) values: HashMap<String, CellValue>,

    /// A hashmap which maps a cell number to its corresponding cell command.
    pub(crate) commands: HashMap<String, String>,
}

impl Spreadsheet {
    pub(crate) fn new() -> Self {
        Spreadsheet {
            values: HashMap::new(),
            commands: HashMap::new(),
        }
    }
}
