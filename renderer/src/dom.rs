use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    Element(ElementData),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementData {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
}

impl Node {
    pub fn text(data: String) -> Node {
        Node {
            node_type: NodeType::Text(data),
            children: Vec::new(),
        }
    }

    pub fn element(tag_name: String, attributes: HashMap<String, String>, children: Vec<Node>) -> Node {
        Node {
            node_type: NodeType::Element(ElementData {
                tag_name,
                attributes,
            }),
            children,
        }
    }

    pub fn get_attribute(&self, name: &str) -> Option<&str> {
        match &self.node_type {
            NodeType::Element(elem) => elem.attributes.get(name).map(|s| s.as_str()),
            NodeType::Text(_) => None,
        }
    }

    pub fn get_id(&self) -> Option<&str> {
        self.get_attribute("id")
    }

    pub fn get_classes(&self) -> Vec<&str> {
        match self.get_attribute("class") {
            Some(s) => s.split_whitespace().collect(),
            None => Vec::new(),
        }
    }
}
