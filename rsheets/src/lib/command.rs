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
