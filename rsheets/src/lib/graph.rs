use crate::command::command_variable_finder;
use crate::spreadsheet::Spreadsheet;
use rsheet_lib::cell_value::CellValue;

use rsheet_lib::command_runner::CommandRunner;
use std::collections::HashMap;

pub(crate) fn update_dependencies(
    cell: &str,
    spreadsheet: &Spreadsheet,
) -> (HashMap<String, CellValue>, Vec<String>) {
    let mut res: (HashMap<String, CellValue>, Vec<String>) = spreadsheet
        .cells
        .iter()
        .map(|(key, val)| (key.clone(), val.command.clone()))
        .filter(|(_key, val)| val.contains(cell))
        .fold((HashMap::new(), Vec::new()), |mut acc, (key, val)| {
            let command = CommandRunner::new(val.as_str());
            match command_variable_finder(&command, &spreadsheet) {
                Ok(val) => {
                    acc.0.insert(key.into(), command.run(&val));
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
