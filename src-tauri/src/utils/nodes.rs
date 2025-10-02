#![allow(unused_imports, dead_code)]
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use xml::name::OwnedName;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    pub name: NodeName,
    pub attributes: Vec<(NodeName, String)>,
    pub namespace: Vec<String>,
    pub childs: Nodes,
}

impl Node {
    pub fn new(
        name: NodeName,
        attributes: Vec<(NodeName, String)>,
        namespace: &BTreeMap<String, String>,
    ) -> Self {
        Self {
            name,
            attributes,
            namespace: namespace
                .iter()
                .filter(|(key, value)| !key.is_empty() && !value.is_empty()) // Filter out empty keys and values
                .map(|(key, _)| key.clone()) // Clone the valid entries
                .collect(),
            childs: Nodes::new(),
        }
    }
    pub fn get_attr(&self, name: &str) -> Option<String> {
        self.attributes
            .iter()
            .find(|(n, _)| n.get_local_name() == name)
            .map(|(_, v)| v.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeName {
    // name including prefix if any
    pub name: String,

    /// A namespace URI, e.g. `http://www.w3.org/2000/xmlns/`.
    pub namespace: Option<String>,
}

impl NodeName {
    pub fn get_local_name(&self) -> String {
        self.name.split(':').last().unwrap().to_string()
    }
}

impl Display for NodeName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(ref ns) = self.namespace {
            write!(f, " (ns: {})", ns)?;
        }
        Ok(())
    }
}

impl From<OwnedName> for NodeName {
    fn from(value: OwnedName) -> Self {
        Self {
            name: match &value.prefix {
                None => value.local_name.clone(),
                Some(prefix) => format!("{}:{}", prefix, value.local_name.clone()),
            },
            namespace: value.namespace,
        }
    }
}

pub type RefC<T> = Rc<RefCell<T>>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nodes {
    pub nodes: Vec<RefC<Node>>,
    pub current_index: usize,
}

impl Nodes {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            current_index: 0,
        }
    }

    pub fn find_node(&self, predicate: impl Fn(Node) -> bool) -> Option<RefC<Node>> {
        let option = self.iter().find(|n| predicate(n.borrow().clone()));
        match option {
            None => self
                .iter()
                .filter(|n| n.borrow().childs.len() > 0)
                .filter_map(|n| n.borrow().childs.find_node(|n| predicate(n.clone())))
                .next(),
            Some(node) => Some(node.clone()),
        }
    }
}

impl Deref for Nodes {
    type Target = Vec<RefC<Node>>;
    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}

impl DerefMut for Nodes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}
