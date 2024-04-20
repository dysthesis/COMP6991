use std::collections::HashMap;

use petgraph::{
    algo::toposort,
    graph::{DiGraph, NodeIndex},
    visit::{Dfs, Walker},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rsheet_lib::{cell_value::CellValue, command_runner::CommandRunner};

use crate::command::{command_variable_finder, list_cells_in_range};

pub(crate) struct Cell {
    pub(crate) value: CellValue,
    pub(crate) command: String,
}

impl Cell {
    pub(crate) fn update(&self, spreadsheet: &Spreadsheet) -> Result<Self, String> {
        Self::new(self.command.clone(), spreadsheet)
    }
    pub(crate) fn new(command: String, spreadsheet: &Spreadsheet) -> Result<Self, String> {
        let runner = CommandRunner::new(command.as_str());
        let variables = match command_variable_finder(&runner, spreadsheet) {
            Ok(variables) => variables,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        let value = runner.run(&variables);

        Ok(Cell { value, command })
    }
}

/// A struct which encapsulates the Spreadsheet itself.
pub(crate) struct Spreadsheet {
    /// A hashmap which stores all of the values in the spreadsheet.
    /// Consists of a key, a String, which represents the cell number,
    /// and a value, the corresponding CellValue
    pub(crate) cells: HashMap<String, Cell>,
    pub(crate) dependency_graph: DiGraph<String, ()>,
    pub(crate) nodes: HashMap<String, NodeIndex>,
}

impl Spreadsheet {
    pub(crate) fn new() -> Self {
        Spreadsheet {
            cells: HashMap::new(),
            dependency_graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }

    pub(crate) fn set(&mut self, key: String, command: String) -> Result<(), String> {
        let cell = Cell::new(command, &self)?;
        self.cells.insert(key.clone(), cell);
        // If this is a new node, add it to the dependency graph

        if !self.nodes.contains_key(&key) {
            let new_node = self.dependency_graph.add_node(key.clone());
            self.nodes.insert(key.clone(), new_node);
        }
        self.update_dependency_graph(key.clone())?;
        self.update_dependents(key)
    }

    pub(crate) fn get(&self, key: String) -> Result<CellValue, String> {
        match self.cells.get(&key) {
            Some(val) => Ok(val.value.clone()),
            None => Err(format!("no cells found for key {key}")),
        }
    }

    fn update_dependency_graph(&mut self, key: String) -> Result<(), String> {
        let cell = match self.cells.get(&key) {
            Some(cell) => cell,
            None => {
                return Err(format!(
                    "can't update dependency graph because no cell is found for key {key}"
                ))
            }
        };
        let command = CommandRunner::new(&cell.command);
        let dependencies: Vec<String> = command
            .find_variables()
            .par_iter()
            .map(|x| dbg!(list_cells_in_range(x)))
            .flatten()
            .flatten()
            .flatten()
            .collect();
        let target = match self.nodes.get(&key) {
            Some(node) => node,
            None => {
                return Err(format!("could not find the node index for cell {key}"));
            }
        };
        // Update the dependency graph
        let errors: Vec<String> = dependencies.iter().fold(Vec::new(), |mut acc, x| {
            match self.nodes.get(x) {
                Some(node) => { self
                    .dependency_graph
                    .add_edge(node.to_owned(), target.to_owned(), ()); },
                None => {acc.push(format!("the dependency {x} for cell {key} does not have a node in the dependency graph"));}
            };
            return acc;
        });

        if !errors.is_empty() {
            return Err(errors
                .first()
                .expect("we checked that `errors` is not empty")
                .to_string());
        }

        Ok(())
    }

    fn update_dependents(&mut self, key: String) -> Result<(), String> {
        let start = match self.nodes.get(&key) {
            Some(index) => index,
            None => {
                return Err(format!("cannot find the node index for cell {key}"));
            }
        };

        let dependents: Vec<NodeIndex> = Dfs::new(&self.dependency_graph, start.to_owned())
            .iter(&self.dependency_graph)
            .collect();

        let subgraph = self.dependency_graph.filter_map(
            |id, node| {
                if dependents.contains(&id) {
                    Some(node.clone())
                } else {
                    None
                }
            },
            |id, edge| {
                let (source, target) = self.dependency_graph.edge_endpoints(id).unwrap();
                if dependents.contains(&source) && dependents.contains(&target) {
                    Some(edge.clone())
                } else {
                    None
                }
            },
        );

        let to_update = match toposort(&subgraph, None) {
            Ok(res) => res,
            Err(e) => {
                let cell_id = subgraph
                    .node_weight(e.node_id())
                    .expect("we can't have a cycle on a nonexistent node");
                return Err(format!("Error: Cycle detected in cell {cell_id}"));
            }
        };

        let errors: Vec<String> = to_update.iter().fold(Vec::new(), |mut acc, node| {
            let cell_id = match self.dependency_graph.node_weight(*node) {
                Some(id) => id,
                None => {
                    let index = node.index();
                    acc.push(String::from(format!(
                        "Cannot find the cell ID associated with the node {index}"
                    )));
                    return acc;
                }
            };

            if let Err(e) = self.update_cell(dbg!(cell_id.to_string())) {
                acc.push(e);
            };
            return acc;
        });

        if !errors.is_empty() {
            return Err(errors
                .first()
                .expect("we checked that `errors` is not empty")
                .to_string());
        }

        Ok(())
    }

    fn update_cell(&mut self, key: String) -> Result<(), String> {
        let cell = match self.cells.get(&key) {
            Some(val) => val,
            None => {
                return Err(String::from(format!(
                    "Cannot find cell associated with the key {key}"
                )));
            }
        };
        let updated_cell = cell.update(self)?;
        self.cells.insert(key, updated_cell);
        Ok(())
    }
}
