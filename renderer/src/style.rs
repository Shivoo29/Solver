use crate::css_parser::{parse_color_keyword, Color, Declaration, Rule, Selector, SimpleSelector, Specificity, Stylesheet, Unit, Value};
use crate::dom::{Node, NodeType};
use std::collections::HashMap;

pub type PropertyMap = HashMap<String, Value>;

#[derive(Debug, Clone)]
pub struct StyledNode<'a> {
    pub node: &'a Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Display {
    pub display_type: DisplayType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayType {
    Block,
    Inline,
    None,
}

impl Default for DisplayType {
    fn default() -> Self {
        DisplayType::Block
    }
}

pub fn style_tree<'a>(root: &'a Node, stylesheet: &'a Stylesheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match root.node_type {
            NodeType::Element(ref elem) => specified_values(root, stylesheet),
            NodeType::Text(_) => HashMap::new(),
        },
        children: root
            .children
            .iter()
            .map(|child| style_tree(child, stylesheet))
            .collect(),
    }
}

fn specified_values(elem: &Node, stylesheet: &Stylesheet) -> PropertyMap {
    let mut values = HashMap::new();
    let mut rules = matching_rules(elem, stylesheet);

    // Sort by specificity (lowest to highest, so higher specificity overwrites)
    rules.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }

    values
}

fn matching_rules<'a>(elem: &Node, stylesheet: &'a Stylesheet) -> Vec<(Specificity, &'a Rule)> {
    stylesheet
        .rules
        .iter()
        .filter_map(|rule| match_rule(elem, rule))
        .collect()
}

fn match_rule<'a>(elem: &Node, rule: &'a Rule) -> Option<(Specificity, &'a Rule)> {
    rule.selectors
        .iter()
        .find(|selector| matches_selector(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matches_selector(elem: &Node, selector: &Selector) -> bool {
    match selector {
        Selector::Simple(ref simple) => matches_simple_selector(elem, simple),
    }
}

fn matches_simple_selector(elem: &Node, selector: &SimpleSelector) -> bool {
    // Check tag name
    if let Some(ref tag) = selector.tag_name {
        if let NodeType::Element(ref elem_data) = elem.node_type {
            if &elem_data.tag_name != tag {
                return false;
            }
        } else {
            return false;
        }
    }

    // Check ID
    if let Some(ref id) = selector.id {
        if elem.get_id() != Some(id) {
            return false;
        }
    }

    // Check classes
    let elem_classes = elem.get_classes();
    for class in &selector.classes {
        if !elem_classes.contains(&class.as_str()) {
            return false;
        }
    }

    true
}

impl<'a> StyledNode<'a> {
    pub fn value(&self, name: &str) -> Option<Value> {
        self.specified_values.get(name).cloned()
    }

    pub fn lookup(&self, name: &str, fallback_name: &str, default: &Value) -> Value {
        self.value(name)
            .or_else(|| self.value(fallback_name))
            .unwrap_or_else(|| default.clone())
    }

    pub fn display(&self) -> Display {
        match self.value("display") {
            Some(Value::Keyword(s)) => Display {
                display_type: match s.as_str() {
                    "block" => DisplayType::Block,
                    "none" => DisplayType::None,
                    _ => DisplayType::Inline,
                },
            },
            _ => Display::default(),
        }
    }

    pub fn color(&self) -> Color {
        match self.value("color") {
            Some(Value::Color(c)) => c,
            Some(Value::Keyword(ref k)) => parse_color_keyword(k).unwrap_or(Color::black()),
            _ => Color::black(),
        }
    }

    pub fn background_color(&self) -> Option<Color> {
        match self.value("background-color") {
            Some(Value::Color(c)) => Some(c),
            Some(Value::Keyword(ref k)) => parse_color_keyword(k),
            _ => None,
        }
    }

    pub fn font_size(&self) -> f32 {
        match self.value("font-size") {
            Some(Value::Length(size, Unit::Px)) => size,
            _ => 16.0,
        }
    }
}
