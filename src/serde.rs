use std::collections::HashMap;
use std::error::Error;
use std::{fs::File, path::Path};

use serde_json::Value;

use crate::arenatree::*;
use crate::error::serde::*;
use crate::util::logging::{check_result, fatal, info, non_fatal};

pub struct QASerde {
    tree: Arena<String>,
    pub question_id: Vec<NodeId> 
}

impl QASerde {
    pub fn new() -> Self {

        QASerde { tree: Arena::new(), question_id: vec![] }
    }

    pub fn build(mut self, path: &str) -> Result<Self, Box<dyn Error>> {
        // Читаем JSON-файл
        let path = Path::new(path);
        let file = match File::open(path) {
            Ok(f) => f,
            Err(e) => return Err(Box::new(e)),
        };

        // Парсим JSON
        let v: Value = match serde_json::from_reader(file) {
            Ok(val) => val,
            Err(e)  => return Err(Box::new(e)),
        };

        check_result(self.parse_into_tree(v, None), fatal);

        self.tree.dbg_list_nodes();
        Ok(self)
    } 

    fn parse_into_tree(&mut self, value: Value, parent: Option<NodeId>) -> Result<(), FileFormattingError> {
        match value {
            Value::Object(map) => {
                for (node_data, children) in map {
                    dbg!("Объект");
                    let new_parent = self.tree.add_node(node_data.to_owned(), parent);
                    self.parse_into_tree(children, new_parent)?;
                }
            }
            Value::String(leaf) => {
                dbg!("Строка");
                let _ = self.tree.add_node(leaf, parent);
                self.question_id.push(parent.unwrap_or(0));
            }
            _ => return Err(FileFormattingError),
        }

        Ok(())
    }

    pub fn is_question(&self, node_data: &str) -> bool {
        self.tree.get_leaves_parents().contains(&self.tree.get_id_by_value(node_data.to_owned()).unwrap())
    }

    pub fn get_parent(&self, node_id: NodeId) -> Option<NodeId> {
        self.tree.get_parent(node_id)
    }

    pub fn get_children(&self, name: Option<String>) -> Result<Vec<String>, IndexError<String>> {
        let name = name.unwrap_or(self.tree.get_root_value().ok_or(IndexError { index: "root".to_string() })?);

        if let Some(children_ids) = self.tree.get_children_by_value(name.clone()) {
            Ok(
                children_ids.iter()
                            .map(|&id| (self.tree.get(id)
                                                // TODO некрасиво, что-то сделать
                                                .unwrap()
                                                .clone()))
                            .collect()
            )
        } 
        else {
            Err(IndexError { index: name.to_owned() })
        }
    }

    pub fn contains(&self, node_data: &str) -> bool {
        self.tree.contains(node_data.to_owned())
    }
}
