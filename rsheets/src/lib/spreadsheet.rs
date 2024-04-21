use crate::command::{command_variable_finder, list_cells_in_range};
use dashmap::{DashMap, DashSet};
use log::info;
use parking_lot::Mutex;
use petgraph::{
    algo::{is_cyclic_directed, toposort},
    graph::{DiGraph, NodeIndex},
    visit::{Bfs, Walker},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rsheet_lib::{cell_value::CellValue, command_runner::CommandRunner};
use std::{collections::HashMap, sync::Arc};

pub(crate) struct Cell {
    pub(crate) value: Arc<Mutex<CellValue>>,
    pub(crate) command: Arc<Mutex<String>>,
}

impl Cell {
    pub(crate) fn update(&self, spreadsheet: &Spreadsheet) -> Result<Self, String> {
        Self::new(self.command.clone().lock().clone(), spreadsheet)
    }
    pub(crate) fn new(command: String, spreadsheet: &Spreadsheet) -> Result<Self, String> {
        let runner = CommandRunner::new(command.as_str());
        let variables = match command_variable_finder(&runner, spreadsheet) {
            Ok(variables) => variables,
            Err(e) => {
                return Err(e.to_string());
            }
        };
        let value = Arc::new(Mutex::new(runner.run(&variables)));
        let command = Arc::new(Mutex::new(command));
        Ok(Cell { value, command })
    }
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        Self {
            value: Arc::new(Mutex::new(self.value.clone().lock().clone())),
            command: Arc::new(Mutex::new(self.command.clone().lock().clone())),
        }
    }
}

/// A struct which encapsulates the Spreadsheet itself.
pub(crate) struct Spreadsheet {
    /// A hashmap which stores all of the values in the spreadsheet.
    /// Consists of a key, a String, which represents the cell number,
    /// and a value, the corresponding CellValue
    pub(crate) cells: DashMap<String, Cell>,
    pub(crate) dependency_graph: Arc<Mutex<DiGraph<String, ()>>>,
    pub(crate) nodes: DashMap<String, NodeIndex>,
    pub(crate) invalid_nodes: DashSet<String>,
}

impl Spreadsheet {
    pub(crate) fn new() -> Self {
        Spreadsheet {
            cells: DashMap::new(),
            dependency_graph: Arc::new(Mutex::new(DiGraph::new())),
            nodes: DashMap::new(),
            invalid_nodes: DashSet::new(),
        }
    }

    pub(crate) fn is_self_referential(&self) -> bool {
        is_cyclic_directed(&*self.dependency_graph.clone().lock())
    }

    pub(crate) fn set(&self, key: String, command: String) -> Result<(), String> {
        info!("In Spreadsheet::set(): setting the value for {key} to {command}");
        if !self.nodes.contains_key(&key) {
            info!("Inserting node for key {key} to dependency graph");
            let new_node = self.dependency_graph.clone().lock().add_node(key.clone());
            info!("Inserting node index to hash map");
            self.nodes.insert(key.clone(), new_node);
        }

        let cell = match Cell::new(command, &self) {
            Ok(res) => {
                // Remove cell from the list of invalid nodes if it is there, since we've checked that it's valid. This is fine because it will be re-added by update_dependency_graph if any of its dependencies are invalid anyways.
                if self.invalid_nodes.contains(&key) {
                    info!("The list of invalid nodes contains cell {key}, but we've checked that it is valid. Removing...");
                    self.invalid_nodes.remove(&key);
                }
                res
            }
            Err(e) => {
                info!("Marking cell as invalid");
                self.invalid_nodes.insert(key);
                return Err(e);
            }
        };

        self.cells.insert(key.clone(), cell);
        info!("Successfully inserted the cell to key {}", key);
        // If this is a new node, add it to the dependency graph

        info!("Updating the dependency graph from key {key}");
        self.update_dependency_graph(key.clone())?;
        info!("Dependency graph updated!");
        self.update_dependents(key)
    }

    pub(crate) fn is_invalid_node(&self, key: String) -> bool {
        self.invalid_nodes.contains(&key)
    }

    pub(crate) fn get(&self, key: String) -> Option<CellValue> {
        match self.cells.get(&key) {
            Some(val) => Some(val.value.clone().lock().clone()),
            None => None,
        }
    }

    pub(crate) fn get_values(&self) -> HashMap<String, CellValue> {
        self.cells.iter().fold(HashMap::new(), |mut acc, x| {
            acc.insert(x.key().to_string(), x.value().value.clone().lock().clone());
            acc
        })
    }

    fn update_dependency_graph(&self, key: String) -> Result<(), String> {
        let cell = match self.cells.get(&key) {
            Some(cell) => cell,
            None => {
                return Err(format!(
                    "can't update dependency graph because no cell is found for key {key}"
                ))
            }
        };
        let command = CommandRunner::new(&cell.command.clone().lock());
        let dependencies: Vec<String> = command
            .find_variables()
            .par_iter()
            .map(|x| list_cells_in_range(x))
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

        // Mark self as invalid if any of its dependencies is invalid.
        dependencies.iter().for_each(|x| {
            let cell = match self.cells.get(x) {
                Some(res) => res.clone().value.clone().lock().clone(),
                None => CellValue::None,
            };
            if let CellValue::Error(_) = cell {
                info!(
                    "The cell {x} which {key} depends on is invalid. Marking {key} as invalid..."
                );
                self.invalid_nodes.insert(key.clone());
            }
        });

        // Update the dependency graph
        let errors: Vec<String> = dependencies.iter().fold(Vec::new(), |mut acc, x| {
            match self.nodes.get(x) {
                Some(node) => {
                    self.dependency_graph.clone().lock().add_edge(node.to_owned(), target.to_owned(), ());
                },
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

        if is_cyclic_directed(&*self.dependency_graph.clone().lock()) {
            return Err(String::from(format!("Graph is self-referentiial")));
        }

        Ok(())
    }

    fn update_dependents(&self, key: String) -> Result<(), String> {
        info!("Updating the dependents of cell {key}");
        let start = match self.nodes.get(&key) {
            Some(index) => index,
            None => {
                return Err(format!("cannot find the node index for cell {key}"));
            }
        };
        info!("Found node index associated with the cell");

        let dependency_graph = self.dependency_graph.clone().lock().clone();
        let dependents: Vec<NodeIndex> = Bfs::new(&dependency_graph, start.to_owned())
            .iter(&dependency_graph)
            .collect();

        info!("Found list of cells dependent on {key}");

        let subgraph = dependency_graph.filter_map(
            |id, node| {
                if dependents.contains(&id) {
                    Some(node.clone())
                } else {
                    None
                }
            },
            |id, edge| {
                let (source, target) = self
                    .dependency_graph
                    .clone()
                    .lock()
                    .edge_endpoints(id)
                    .unwrap();
                if dependents.contains(&source) && dependents.contains(&target) {
                    Some(edge.clone())
                } else {
                    None
                }
            },
        );
        info!("Constructed a subgraph of dependents for cell");

        let to_update = match toposort(&subgraph, None) {
            Ok(res) => res,
            Err(e) => {
                let cell_id = subgraph
                    .node_weight(e.node_id())
                    .expect("we can't have a cycle on a nonexistent node");
                self.invalid_nodes.insert(cell_id.to_string());
                info!("Inserted {cell_id} to the list of invalid nodes due to cycle detected.");
                return Err(format!("Error: Cycle detected in cell {cell_id}"));
            }
        };
        info!("Topologically sorted the dependent cells, proceeding to update their values...");

        let errors: Vec<String> = to_update.iter().fold(Vec::new(), |mut acc, node| {
            let cell_id = match subgraph.node_weight(*node) {
                Some(id) => id,
                None => {
                    let index = node.index();
                    acc.push(String::from(format!(
                        "Cannot find the cell ID associated with the node {index}"
                    )));
                    return acc;
                }
            };
            info!("Updating the value for cell {cell_id}");
            if let Err(e) = self.update_cell(cell_id.to_string()) {
                info!("Failed to update value for cell {cell_id}");
                acc.push(e);
            };
            return acc;
        });

        info!("Updated the values for cell dependents, checking for errors...");
        if !errors.is_empty() {
            return Err(errors
                .first()
                .expect("we checked that `errors` is not empty")
                .to_string());
        }

        info!("Successfully updated dependent cells!");
        Ok(())
    }

    fn update_cell(&self, key: String) -> Result<(), String> {
        info!("Updating the value for cell {key}.");
        let cell = match self.cells.get(&key) {
            Some(val) => val.clone(),
            None => {
                return Err(String::from(format!(
                    "Cannot find cell associated with the key {key}"
                )));
            }
        };
        info!("Found the relevant cell to update.");
        let updated_cell = cell.update(self)?;
        info!("Obtained new, updated copy of the cell.");
        self.cells.insert(key, updated_cell);
        info!("Inserted the new updated cell");
        Ok(())
    }
}
