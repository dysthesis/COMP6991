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
