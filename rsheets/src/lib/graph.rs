use crate::command::command_variable_finder;
use crate::spreadsheet::Spreadsheet;
use rsheet_lib::cell_value::CellValue;
use rsheet_lib::cells::column_name_to_number;
use rsheet_lib::cells::column_number_to_name;
use rsheet_lib::command_runner::CommandRunner;
use std::collections::HashMap;

pub(crate) fn update_dependencies(
    cell: &str,
    spreadsheet: &Spreadsheet,
) -> (HashMap<String, CellValue>, Vec<String>) {
    let mut res: (HashMap<String, CellValue>, Vec<String>) = spreadsheet
        .commands
        .iter()
        .filter(|(_key, val)| val.contains(cell))
        .fold((HashMap::new(), Vec::new()), |mut acc, (key, val)| {
            let command = CommandRunner::new(val);
            match command_variable_finder(&command, &spreadsheet) {
                Ok(val) => {
                    acc.0.insert(key.into(), command.run(&dbg!(val)));
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

pub(crate) fn cells_to_value(
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
