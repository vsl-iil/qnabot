use std::{borrow::Borrow, fmt::Debug};

pub struct Arena<T: Debug> {
    nodes: Vec<Node<T>>,
}

pub type NodeId = usize;

#[derive(Debug)]
pub struct Node<T: Debug> {
    parent: Option<NodeId>,
    children: Vec<NodeId>,

    pub data: T,
}

impl<Borrowed: Debug> Borrow<Borrowed> for Node<Borrowed> {
    fn borrow(&self) -> &Borrowed {
        return &self.data
    }
}

impl<T: Eq + Clone + Debug> Arena<T> {
    pub fn new() -> Self {
        Arena { nodes: vec![] }
    }

    pub fn dbg_list_nodes(&self) {
        println!("Ноды:");
        for node in &self.nodes {
            print!("Нода: ");
            println!("{node:?}");
        }
    }

    pub fn add_node(&mut self, data: T, parent: Option<NodeId>) -> Option<NodeId> {
        dbg!("Добавляем ноду...");
        let new_index = self.nodes.len();

        if let Some(parent_id) = parent {
            dbg!("Есть родитель...");
            if let Some(parent_node) = self.nodes.get_mut(parent_id) {
                parent_node.children.push(new_index);

                dbg!("Добавляем в массив");
                self.nodes.push(Node {
                    parent: Some(parent_id), 
                    children: vec![], 
                    data 
                });

                Some(new_index)
            } else { None }     // Нет такого родителя
        } else {                // Создаем корень
            self.nodes.push(Node {
                parent: None,
                children: vec![],
                data
            });

            Some(0)
        }         

    }

    pub fn get_parent(&self, node_id: NodeId) -> Option<NodeId> {
        if let Some(node) = self.nodes.get(node_id) {
            return node.parent
        } else {
            None
        }
    }

    pub fn get_children_by_value(&self, node_data: T) -> Option<Vec<NodeId>> {
        self.nodes.iter()
                  .find(|node| node.data == node_data)
                  .map( |node| node.children.clone())
    }

    pub fn get_leaves_parents(&self) -> Vec<NodeId> {
        self.nodes.iter()
                  .filter(|node| node.children.is_empty())
                  .map( |node| node.parent.unwrap())
                  .collect()
    }

    pub fn get_children_by_id(&self, node_id: NodeId) -> Option<Vec<NodeId>> {
        if let Some(node) = self.nodes.get(node_id) {
            Some(node.children.clone())
        } else {
            None
        }
    }

    pub fn get_id_by_value(&self, node_data: T) -> Option<NodeId> {
        self.nodes.iter()
                  .enumerate()
                  .find(|(_, node)| node.data == node_data)
                  .map( |(index,_)| index)
    }

    pub fn get_root_value(&self) -> Option<T> {
        if let Some(root) = self.nodes.iter().find(|n| n.parent.is_none()) {
            Some(root.data.clone())
        } else {
            None
        }
    }

    pub fn get(&self, node_id: NodeId) -> Option<&T> {
        if let Some(node) = self.nodes.get(node_id) {
            Some(node.borrow())
        } else {
            None
        }
    }

    pub fn contains(&self, node_data: T) -> bool {
        for val in &self.nodes {
            if val.data == node_data {
                return true;
            }
        }

        return false;
    }
}
