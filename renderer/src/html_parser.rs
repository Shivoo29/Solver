use crate::dom::{Node, NodeType, ElementData};
use std::collections::HashMap;

pub struct HtmlParser {
    pos: usize,
    input: String,
}

impl HtmlParser {
    pub fn parse(source: String) -> Node {
        let mut parser = HtmlParser {
            pos: 0,
            input: source,
        };
        let nodes = parser.parse_nodes();

        // If there's only one root node, return it; otherwise wrap in a div
        if nodes.len() == 1 {
            nodes.into_iter().next().unwrap()
        } else {
            Node::element("div".to_string(), HashMap::new(), nodes)
        }
    }

    fn parse_nodes(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            self.consume_whitespace();
            if self.eof() || self.starts_with("</") {
                break;
            }
            nodes.push(self.parse_node());
        }
        nodes
    }

    fn parse_node(&mut self) -> Node {
        if self.next_char() == '<' {
            // Check for special cases
            if self.starts_with("<!DOCTYPE") || self.starts_with("<!doctype") {
                self.skip_doctype();
                // Try parsing next node
                if !self.eof() {
                    return self.parse_node();
                } else {
                    return Node::text("".to_string());
                }
            } else if self.starts_with("<!--") {
                self.skip_comment();
                // Try parsing next node
                if !self.eof() {
                    return self.parse_node();
                } else {
                    return Node::text("".to_string());
                }
            } else {
                self.parse_element()
            }
        } else {
            self.parse_text()
        }
    }

    fn skip_doctype(&mut self) {
        // Skip until we find '>'
        while !self.eof() && self.next_char() != '>' {
            self.consume_char();
        }
        if !self.eof() {
            self.consume_char(); // consume '>'
        }
    }

    fn skip_comment(&mut self) {
        // Skip <!--
        self.consume_char(); // <
        self.consume_char(); // !
        self.consume_char(); // -
        self.consume_char(); // -

        // Skip until we find -->
        while !self.eof() {
            if self.starts_with("-->") {
                self.consume_char(); // -
                self.consume_char(); // -
                self.consume_char(); // >
                break;
            }
            self.consume_char();
        }
    }

    fn parse_element(&mut self) -> Node {
        // Opening tag
        assert_eq!(self.consume_char(), '<');
        let tag_name = self.parse_tag_name();
        let attributes = self.parse_attributes();
        assert_eq!(self.consume_char(), '>');

        // Self-closing tags
        let self_closing = matches!(
            tag_name.as_str(),
            "br" | "hr" | "img" | "input" | "meta" | "link"
        );

        if self_closing {
            return Node::element(tag_name, attributes, Vec::new());
        }

        // Contents
        let children = self.parse_nodes();

        // Closing tag
        if self.starts_with("</") {
            assert_eq!(self.consume_char(), '<');
            assert_eq!(self.consume_char(), '/');
            let close_tag = self.parse_tag_name();
            // Be lenient about mismatched tags
            if close_tag != tag_name {
                eprintln!("Warning: mismatched tags {} and {}", tag_name, close_tag);
            }
            assert_eq!(self.consume_char(), '>');
        }

        Node::element(tag_name, attributes, children)
    }

    fn parse_tag_name(&mut self) -> String {
        self.consume_while(|c| c.is_alphanumeric() || c == '-')
    }

    fn parse_attributes(&mut self) -> HashMap<String, String> {
        let mut attributes = HashMap::new();
        loop {
            self.consume_whitespace();
            if self.next_char() == '>' {
                break;
            }
            let (name, value) = self.parse_attribute();
            attributes.insert(name, value);
        }
        attributes
    }

    fn parse_attribute(&mut self) -> (String, String) {
        let name = self.parse_tag_name();
        self.consume_whitespace();

        let value = if self.next_char() == '=' {
            self.consume_char();
            self.consume_whitespace();
            self.parse_attribute_value()
        } else {
            String::new()
        };

        (name, value)
    }

    fn parse_attribute_value(&mut self) -> String {
        let quote = self.next_char();
        if quote == '"' || quote == '\'' {
            self.consume_char();
            let value = self.consume_while(|c| c != quote);
            assert_eq!(self.consume_char(), quote);
            value
        } else {
            self.consume_while(|c| !c.is_whitespace() && c != '>')
        }
    }

    fn parse_text(&mut self) -> Node {
        let text = self.consume_while(|c| c != '<');
        Node::text(text)
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while !self.eof() && test(self.next_char()) {
            result.push(self.consume_char());
        }
        result
    }

    fn consume_char(&mut self) -> char {
        let mut iter = self.input[self.pos..].char_indices();
        let (_, cur_char) = iter.next().unwrap();
        let (next_pos, _) = iter.next().unwrap_or((1, ' '));
        self.pos += next_pos;
        cur_char
    }

    fn next_char(&self) -> char {
        self.input[self.pos..].chars().next().unwrap_or('\0')
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.pos..].starts_with(s)
    }

    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let html = r#"<html><body><h1>Hello</h1></body></html>"#;
        let node = HtmlParser::parse(html.to_string());
        match node.node_type {
            NodeType::Element(ref elem) => {
                assert_eq!(elem.tag_name, "html");
            }
            _ => panic!("Expected element"),
        }
    }
}
