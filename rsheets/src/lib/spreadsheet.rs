use crate::command::{command_variable_finder, list_cells_in_range};
use dashmap::{DashMap, DashSet};
use log::info;
use parking_lot::Mutex;
use petgraph::{
    algo::{is_cyclic_directed, toposort},
    graph::{DiGraph, NodeIndex},
    visit::{Bfs, EdgeRef, Walker},
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rsheet_lib::{cell_value::CellValue, command_runner::CommandRunner};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

pub(crate) struct Cell {
    pub(crate) value: Arc<Mutex<CellValue>>,
    pub(crate) command: Arc<Mutex<String>>,
}

impl Cell {
    pub(crate) fn update(&self, spreadsheet: &Spreadsheet) -> Self {
        Self::new(self.command.lock().clone(), spreadsheet)
    }
    pub(crate) fn new(command: String, spreadsheet: &Spreadsheet) -> Self {
        let runner = CommandRunner::new(command.as_str());
        let variables = match command_variable_finder(&runner, spreadsheet) {
            Ok(variables) => variables,
            Err(_) => {
                let value = Arc::new(Mutex::new(CellValue::None));
                let command = Arc::new(Mutex::new(command));
                return Cell { value, command };
            }
        };
        let value = Arc::new(Mutex::new(runner.run(&variables)));
        let command = Arc::new(Mutex::new(command));
        Cell { value, command }
    }
}

impl Clone for Cell {
    fn clone(&self) -> Self {
        Self {
            value: Arc::new(Mutex::new(self.value.lock().clone())),
            command: Arc::new(Mutex::new(self.command.lock().clone())),
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
        is_cyclic_directed(&*self.dependency_graph.lock())
    }

    pub(crate) fn set(&self, key: String, command: String) -> Result<(), String> {
        info!("In Spreadsheet::set(): setting the value for {key} to {command}");
        if !self.nodes.contains_key(&key) {
            info!("Inserting node for key {key} to dependency graph");
            let new_node = self.dependency_graph.lock().add_node(key.clone());
            info!("Inserting node index to hash map");
            self.nodes.insert(key.clone(), new_node);
        }

        let cell = Cell::new(command, &self);

        self.cells.insert(key.clone(), cell);
        info!("Successfully inserted the cell to key {}", key);

        Ok(())
    }

    pub(crate) fn update_dependencies(&self, key: &String) {
        info!("Updating the dependency graph from key {key}");
        let _ = self.update_dependency_graph(key.clone());

        if !self.is_self_referential() {
            info!("We've checked that the graph is not self-referential. Clearing the set of invalid nodes...");
            self.invalid_nodes.clear();
        }

        info!("Dependency graph updated!");
        let _ = self.update_dependents(key.clone());
    }

    pub(crate) fn is_invalid_node(&self, key: String) -> bool {
        self.invalid_nodes.contains(&key)
    }

    pub(crate) fn get(&self, key: String) -> Option<CellValue> {
        match self.cells.get(&key) {
            Some(val) => Some(val.value.lock().clone()),
            None => None,
        }
    }

    pub(crate) fn get_values(&self) -> HashMap<String, CellValue> {
        self.cells.iter().fold(HashMap::new(), |mut acc, x| {
            acc.insert(x.key().to_string(), x.value().value.lock().clone());
            acc
        })
    }

    fn get_dependencies(&self, key: &String) -> Vec<String> {
        let cell = match self.cells.get(key) {
            Some(cell) => cell,
            None => {
                return Vec::new();
            }
        };
        let command = CommandRunner::new(&cell.command.lock());
        command
            .find_variables()
            .par_iter()
            .map(|x| list_cells_in_range(x))
            .flatten()
            .flatten()
            .flatten()
            .collect()
    }

    pub(crate) fn has_invalid_dependencies(&self, key: &String) -> bool {
        let dependencies = self.get_dependencies(key);
        let error_dependencies: i32 = dependencies
            .iter()
            .filter_map(|dep| self.cells.get(dep))
            .fold(0, |mut acc, x| {
                if let CellValue::Error(_) = x.value.lock().clone() {
                    acc += 1;
                }
                acc
            });
        error_dependencies != 0i32
    }

    fn update_dependency_graph(&self, key: String) -> Result<(), String> {
        info!("Hello from Spreadsheet::update_dependency_graph!");

        let dependencies = self.get_dependencies(&key);
        info!(
            "Found the list of dependencies for cell {}: {:?}",
            key, dependencies
        );

        let target = match self.nodes.get(&key) {
            Some(node) => node,
            None => {
                return Err(format!("could not find the node index for cell {key}"));
            }
        };
        info!("Found the node for that dependency");
        let mut graph = self.dependency_graph.lock();
        let existing_dependencies: HashSet<NodeIndex> = graph
            .edges_directed(target.to_owned(), petgraph::Direction::Incoming)
            .map(|edge| edge.source())
            .collect();

        // get rid of obsolete dependencies
        existing_dependencies
            .iter()
            .for_each(|dependency| if !dependencies.contains(graph.node_weight(*dependency).expect("we obtained the set of nodes by previously iterating through existing edges, and the graph is locked since then")) {
                info!("The dependency {} is obsolete. Scrubbing...", graph.node_weight(*dependency).expect("if it worked above, it should work here"));
                let edge = graph.find_edge(*dependency, *target).expect("the `existing_dependencies` set is constructed by iterating through existing edges");
                graph.remove_edge(edge);
            });
        info!("Scrubbed obsolete dependencies");

        // Update the dependency graph
        dependencies.iter().for_each(|x| {
            match self.nodes.get(x) {
                Some(node) => {
                    if !existing_dependencies.contains(&node.to_owned()) {
                        graph.add_edge(node.to_owned(), target.to_owned(), ());
                    }
                }
                None => {
                    let new_node = graph.add_node(x.clone());
                    graph.add_edge(new_node, target.to_owned(), ());
                    self.nodes.insert(x.clone(), new_node);
                }
            };
        });

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

        info!("Constructing subgraph of dependents");
        let subgraph = {
            let dependency_graph = self.dependency_graph.lock().clone();
            let dependents: Vec<NodeIndex> = Bfs::new(&dependency_graph, start.to_owned())
                .iter(&dependency_graph)
                .collect();

            dependency_graph.filter_map(
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
                        Some(*edge)
                    } else {
                        None
                    }
                },
            )
        };
        info!("Constructed a subgraph of dependents for cell");

        let to_update = match toposort(&subgraph, None) {
            Ok(res) => {
                if self.invalid_nodes.contains(&key) {
                    self.invalid_nodes.remove(&key);
                }
                res
            }
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
                    acc.push(format!(
                        "Cannot find the cell ID associated with the node {index}"
                    ));
                    return acc;
                }
            };
            info!("Updating the value for cell {cell_id}");
            if let Err(e) = self.update_cell(cell_id.to_string()) {
                info!("Failed to update value for cell {cell_id}");
                acc.push(e);
            };
            acc
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
                return Err(format!("Cannot find cell associated with the key {key}"));
            }
        };
        info!("Found the relevant cell to update.");
        let updated_cell = cell.update(self);
        info!("Obtained new, updated copy of the cell.");
        self.cells.insert(key, updated_cell);
        info!("Inserted the new updated cell");
        Ok(())
    }
}
