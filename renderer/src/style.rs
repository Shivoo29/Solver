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
    Table,
    TableRow,
    TableCell,
    Flex,
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
            Some(Value::Keyword(s)) => {
                let display_type = match s.as_str() {
                    "block" => DisplayType::Block,
                    "inline" => DisplayType::Inline,
                    "none" => DisplayType::None,
                    "table" => DisplayType::Table,
                    "table-row" => DisplayType::TableRow,
                    "table-cell" => DisplayType::TableCell,
                    "flex" => DisplayType::Flex,
                    _ => return Display { display_type: self.default_display_from_tag() },
                };
                Display { display_type }
            },
            _ => Display { display_type: self.default_display_from_tag() },
        }
    }

    fn default_display_from_tag(&self) -> DisplayType {
        // Automatically assign display type based on HTML tag if no CSS display is set
        if let NodeType::Element(ref elem) = self.node.node_type {
            match elem.tag_name.as_str() {
                "table" => DisplayType::Table,
                "tr" => DisplayType::TableRow,
                "td" | "th" => DisplayType::TableCell,
                "div" | "section" | "article" | "header" | "footer" | "main" |
                "nav" | "aside" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" |
                "p" | "ul" | "ol" | "li" | "form" | "fieldset" => DisplayType::Block,
                "span" | "a" | "strong" | "em" | "b" | "i" | "code" => DisplayType::Inline,
                _ => DisplayType::Block,
            }
        } else {
            DisplayType::Inline // Text nodes are inline
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

    // Flexbox properties
    pub fn flex_direction(&self) -> FlexDirection {
        match self.value("flex-direction") {
            Some(Value::Keyword(s)) => match s.as_str() {
                "row" => FlexDirection::Row,
                "row-reverse" => FlexDirection::RowReverse,
                "column" => FlexDirection::Column,
                "column-reverse" => FlexDirection::ColumnReverse,
                _ => FlexDirection::Row,
            },
            _ => FlexDirection::Row,
        }
    }

    pub fn justify_content(&self) -> JustifyContent {
        match self.value("justify-content") {
            Some(Value::Keyword(s)) => match s.as_str() {
                "flex-start" => JustifyContent::FlexStart,
                "flex-end" => JustifyContent::FlexEnd,
                "center" => JustifyContent::Center,
                "space-between" => JustifyContent::SpaceBetween,
                "space-around" => JustifyContent::SpaceAround,
                _ => JustifyContent::FlexStart,
            },
            _ => JustifyContent::FlexStart,
        }
    }

    pub fn align_items(&self) -> AlignItems {
        match self.value("align-items") {
            Some(Value::Keyword(s)) => match s.as_str() {
                "flex-start" => AlignItems::FlexStart,
                "flex-end" => AlignItems::FlexEnd,
                "center" => AlignItems::Center,
                "stretch" => AlignItems::Stretch,
                _ => AlignItems::Stretch,
            },
            _ => AlignItems::Stretch,
        }
    }

    pub fn flex_grow(&self) -> f32 {
        match self.value("flex-grow") {
            Some(Value::Length(n, _)) => n,
            _ => 0.0,
        }
    }

    pub fn flex_shrink(&self) -> f32 {
        match self.value("flex-shrink") {
            Some(Value::Length(n, _)) => n,
            _ => 1.0,
        }
    }

    pub fn flex_basis(&self) -> Option<f32> {
        match self.value("flex-basis") {
            Some(Value::Length(n, Unit::Px)) => Some(n),
            Some(Value::Keyword(s)) if s == "auto" => None,
            _ => None,
        }
    }
}

// Flexbox enums
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlignItems {
    FlexStart,
    FlexEnd,
    Center,
    Stretch,
}
