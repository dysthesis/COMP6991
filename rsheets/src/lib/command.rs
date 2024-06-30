use crate::spreadsheet::Spreadsheet;
use rsheet_lib::{
    cell_value::CellValue,
    cells::{column_name_to_number, column_number_to_name},
    command_runner::{CellArgument, CommandRunner},
};
use std::collections::HashMap;

pub(crate) fn command_variable_finder(
    command: &CommandRunner,
    spreadsheet: &Spreadsheet,
) -> Result<HashMap<String, CellArgument>, &'static str> {
    let referenced: Vec<String> = command.find_variables();

    referenced.iter().try_fold(HashMap::new(), |mut acc, key| {
                            if key.contains('_') {
                                let cell_names = list_cells_in_range(key)?;
                                let cell_values: HashMap<String, CellValue> = spreadsheet.get_values();
                                let values_matrix = cells_to_value(cell_names, &cell_values)?;
                                if values_matrix.len() == 1 {
                                    acc.insert(
                                        key.clone(),
                                        CellArgument::Vector(values_matrix.first().expect("If we have made it this far, the matrix should have at least one vector.").to_owned()),
                                    );
                                } else {
                                    acc.insert(key.clone(), CellArgument::Matrix(values_matrix));
                                }
                            } else if let Some(cell) = spreadsheet.cells.get(key) {
                                acc.insert(key.clone(), CellArgument::Value(cell.value.clone().lock().clone()));
                            } else {
                                return Err("Missing key for individual cell reference");
                            }
                            Ok(acc)
                        })
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
                        .ok_or("Missing key in HashMap")
                })
                .collect::<Result<Vec<_>, _>>() // Collect inner vector results, short-circuiting on error
        })
        .collect() // Collect outer vector results, short-circuiting on error
}

pub(crate) fn list_cells_in_range(range: &str) -> Result<Vec<Vec<String>>, &'static str> {
    if range.chars().all(char::is_alphanumeric) {
        return Ok(vec![vec![String::from(range)]]);
    }
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
