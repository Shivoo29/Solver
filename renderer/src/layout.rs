use crate::css_parser::{Unit, Value};
use crate::style::{DisplayType, StyledNode};
use crate::images::{ImageData, ImageCache};
use crate::dom::NodeType;

#[derive(Debug, Clone, Copy, Default)]
pub struct Dimensions {
    pub content: Rect,
    pub padding: EdgeSizes,
    pub border: EdgeSizes,
    pub margin: EdgeSizes,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EdgeSizes {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Debug)]
pub struct LayoutBox<'a> {
    pub dimensions: Dimensions,
    pub box_type: BoxType<'a>,
    pub children: Vec<LayoutBox<'a>>,
}

#[derive(Debug)]
pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    ImageNode(&'a StyledNode<'a>, Option<ImageData>),
    AnonymousBlock,
}

impl Dimensions {
    pub fn padding_box(&self) -> Rect {
        self.content.expanded_by(&self.padding)
    }

    pub fn border_box(&self) -> Rect {
        self.padding_box().expanded_by(&self.border)
    }

    pub fn margin_box(&self) -> Rect {
        self.border_box().expanded_by(&self.margin)
    }
}

impl Rect {
    pub fn expanded_by(&self, edge: &EdgeSizes) -> Rect {
        Rect {
            x: self.x - edge.left,
            y: self.y - edge.top,
            width: self.width + edge.left + edge.right,
            height: self.height + edge.top + edge.bottom,
        }
    }
}

pub fn layout_tree<'a>(
    node: &'a StyledNode<'a>,
    mut containing_block: Dimensions,
    image_cache: &ImageCache,
) -> LayoutBox<'a> {
    containing_block.content.height = 0.0;

    let mut root_box = build_layout_tree(node, image_cache);
    root_box.layout(containing_block);
    root_box
}

fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>, image_cache: &ImageCache) -> LayoutBox<'a> {
    // Check if this is an <img> element
    let box_type = if let NodeType::Element(elem) = &style_node.node.node_type {
        if elem.tag_name == "img" {
            // Try to load the image
            eprintln!("Found <img> tag");
            let image_data = elem.attributes.get("src")
                .and_then(|src| {
                    eprintln!("Loading image from: {}", src);
                    match image_cache.load_image(src) {
                        Ok(img) => {
                            eprintln!("Image loaded successfully: {}x{}", img.width, img.height);
                            Some(img)
                        }
                        Err(e) => {
                            eprintln!("Failed to load image: {}", e);
                            None
                        }
                    }
                });

            BoxType::ImageNode(style_node, image_data)
        } else {
            match style_node.display().display_type {
                DisplayType::Block => BoxType::BlockNode(style_node),
                DisplayType::Inline => BoxType::InlineNode(style_node),
                DisplayType::None => panic!("Root node has display: none"),
            }
        }
    } else {
        match style_node.display().display_type {
            DisplayType::Block => BoxType::BlockNode(style_node),
            DisplayType::Inline => BoxType::InlineNode(style_node),
            DisplayType::None => panic!("Root node has display: none"),
        }
    };

    let mut root = LayoutBox::new(box_type);

    for child in &style_node.children {
        match child.display().display_type {
            DisplayType::Block => root.children.push(build_layout_tree(child, image_cache)),
            DisplayType::Inline => root
                .get_inline_container()
                .children
                .push(build_layout_tree(child, image_cache)),
            DisplayType::None => {}
        }
    }

    root
}

impl<'a> LayoutBox<'a> {
    fn new(box_type: BoxType<'a>) -> LayoutBox<'a> {
        LayoutBox {
            box_type,
            dimensions: Default::default(),
            children: Vec::new(),
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineNode(_) | BoxType::AnonymousBlock | BoxType::ImageNode(_, _) => self,
            BoxType::BlockNode(_) => {
                match self.children.last() {
                    Some(&LayoutBox {
                        box_type: BoxType::AnonymousBlock,
                        ..
                    }) => {}
                    _ => self.children.push(LayoutBox::new(BoxType::AnonymousBlock)),
                }
                self.children.last_mut().unwrap()
            }
        }
    }

    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => self.layout_inline(containing_block),
            BoxType::ImageNode(_, _) => self.layout_image(containing_block),
        }
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        self.calculate_block_width(containing_block);
        self.calculate_block_position(containing_block);
        self.layout_block_children();
        self.calculate_block_height();
    }

    fn layout_inline(&mut self, containing_block: Dimensions) {
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y + containing_block.content.height;
        self.dimensions.content.width = containing_block.content.width;

        for child in &mut self.children {
            child.layout(self.dimensions);
        }

        self.dimensions.content.height = self
            .children
            .iter()
            .map(|child| child.dimensions.margin_box().height)
            .sum();
    }

    fn layout_image(&mut self, containing_block: Dimensions) {
        // Set position
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y + containing_block.content.height;

        // Get image dimensions
        if let BoxType::ImageNode(_, Some(image_data)) = &self.box_type {
            // Use actual image dimensions, but cap at containing block width
            self.dimensions.content.width = image_data.width.min(containing_block.content.width as u32) as f32;
            self.dimensions.content.height = image_data.height as f32;

            eprintln!("Image layout: {}x{} (original: {}x{})",
                self.dimensions.content.width, self.dimensions.content.height,
                image_data.width, image_data.height);
        } else {
            // No image data, use small default
            self.dimensions.content.width = 10.0;
            self.dimensions.content.height = 10.0;
        }
    }

    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        let auto = Value::Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());

        let zero = Value::Length(0.0, Unit::Px);

        let mut margin_left = style.lookup("margin-left", "margin", &zero);
        let mut margin_right = style.lookup("margin-right", "margin", &zero);

        let border_left = style.lookup("border-left-width", "border-width", &zero);
        let border_right = style.lookup("border-right-width", "border-width", &zero);

        let padding_left = style.lookup("padding-left", "padding", &zero);
        let padding_right = style.lookup("padding-right", "padding", &zero);

        let total: f32 = [
            &margin_left,
            &margin_right,
            &border_left,
            &border_right,
            &padding_left,
            &padding_right,
            &width,
        ]
        .iter()
        .map(|v| v.to_px())
        .sum();

        if width != auto && total > containing_block.content.width {
            if margin_left == auto {
                margin_left = Value::Length(0.0, Unit::Px);
            }
            if margin_right == auto {
                margin_right = Value::Length(0.0, Unit::Px);
            }
        }

        let underflow = containing_block.content.width - total;

        match (width == auto, margin_left == auto, margin_right == auto) {
            (false, false, false) => {
                margin_right = Value::Length(margin_right.to_px() + underflow, Unit::Px);
            }
            (false, false, true) => {
                margin_right = Value::Length(underflow, Unit::Px);
            }
            (false, true, false) => {
                margin_left = Value::Length(underflow, Unit::Px);
            }
            (true, _, _) => {
                if margin_left == auto {
                    margin_left = Value::Length(0.0, Unit::Px);
                }
                if margin_right == auto {
                    margin_right = Value::Length(0.0, Unit::Px);
                }

                if underflow >= 0.0 {
                    width = Value::Length(underflow, Unit::Px);
                } else {
                    width = Value::Length(0.0, Unit::Px);
                    margin_right = Value::Length(margin_right.to_px() + underflow, Unit::Px);
                }
            }
            (false, true, true) => {
                margin_left = Value::Length(underflow / 2.0, Unit::Px);
                margin_right = Value::Length(underflow / 2.0, Unit::Px);
            }
        }

        let d = &mut self.dimensions;
        d.content.width = width.to_px();
        d.padding.left = padding_left.to_px();
        d.padding.right = padding_right.to_px();
        d.border.left = border_left.to_px();
        d.border.right = border_right.to_px();
        d.margin.left = margin_left.to_px();
        d.margin.right = margin_right.to_px();
    }

    fn calculate_block_position(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();
        let d = &mut self.dimensions;

        let zero = Value::Length(0.0, Unit::Px);

        d.margin.top = style.lookup("margin-top", "margin", &zero).to_px();
        d.margin.bottom = style.lookup("margin-bottom", "margin", &zero).to_px();

        d.border.top = style
            .lookup("border-top-width", "border-width", &zero)
            .to_px();
        d.border.bottom = style
            .lookup("border-bottom-width", "border-width", &zero)
            .to_px();

        d.padding.top = style.lookup("padding-top", "padding", &zero).to_px();
        d.padding.bottom = style.lookup("padding-bottom", "padding", &zero).to_px();

        d.content.x = containing_block.content.x + d.margin.left + d.border.left + d.padding.left;

        d.content.y = containing_block.content.y
            + containing_block.content.height
            + d.margin.top
            + d.border.top
            + d.padding.top;
    }

    fn layout_block_children(&mut self) {
        let d = &mut self.dimensions;
        for child in &mut self.children {
            child.layout(*d);
            d.content.height += child.dimensions.margin_box().height;
        }
    }

    fn calculate_block_height(&mut self) {
        if let Some(Value::Length(h, Unit::Px)) = self.get_style_node().value("height") {
            self.dimensions.content.height = h;
        }
    }

    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) | BoxType::ImageNode(node, _) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block has no style node"),
        }
    }
}

impl Value {
    pub fn to_px(&self) -> f32 {
        match self {
            Value::Length(f, Unit::Px) => *f,
            _ => 0.0,
        }
    }
}
