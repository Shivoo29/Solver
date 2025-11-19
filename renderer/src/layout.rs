use crate::css_parser::{Unit, Value};
use crate::style::{DisplayType, StyledNode, FlexDirection, JustifyContent, AlignItems, GridTrackSize, Position};
use crate::images::{ImageData, ImageCache};
use crate::canvas::CanvasRenderingContext2D;
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

#[derive(Debug, Clone)]
pub enum FormElementType {
    TextInput,
    Password,
    Email,
    Number,
    Button,
    Submit,
    Textarea,
    Checkbox,
    Radio,
}

#[derive(Debug, Clone)]
pub struct FormElementData {
    pub element_type: FormElementType,
    pub value: String,
    pub placeholder: String,
    pub name: String,
    pub input_index: Option<usize>, // Index for tracking focus/state
}

#[derive(Debug, Clone)]
pub struct CanvasData {
    pub id: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    ImageNode(&'a StyledNode<'a>, Option<ImageData>),
    FormElement(&'a StyledNode<'a>, FormElementData),
    CanvasNode(&'a StyledNode<'a>, CanvasData),
    TableNode(&'a StyledNode<'a>),
    TableRowNode(&'a StyledNode<'a>),
    TableCellNode(&'a StyledNode<'a>),
    FlexNode(&'a StyledNode<'a>),
    GridNode(&'a StyledNode<'a>),
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

use std::collections::HashMap;

pub fn layout_tree<'a>(
    node: &'a StyledNode<'a>,
    mut containing_block: Dimensions,
    image_cache: &ImageCache,
    form_values: Option<&HashMap<usize, String>>,
) -> LayoutBox<'a> {
    containing_block.content.height = 0.0;

    let mut input_counter = 0;
    let mut root_box = build_layout_tree(node, image_cache, form_values, &mut input_counter);
    root_box.layout(containing_block);

    // Apply CSS positioning after normal layout
    root_box.apply_positioning();

    root_box
}

fn build_layout_tree<'a>(
    style_node: &'a StyledNode<'a>,
    image_cache: &ImageCache,
    form_values: Option<&HashMap<usize, String>>,
    input_counter: &mut usize,
) -> LayoutBox<'a> {
    // Check if this is a form element, image, or regular element
    let box_type = if let NodeType::Element(elem) = &style_node.node.node_type {
        // Check for form elements first
        if elem.tag_name == "input" || elem.tag_name == "button" || elem.tag_name == "textarea" {
            let form_data = create_form_element_data(elem, form_values, input_counter);
            BoxType::FormElement(style_node, form_data)
        } else if elem.tag_name == "img" {
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
        } else if elem.tag_name == "canvas" {
            // Create canvas element
            let id = elem.attributes.get("id").cloned().unwrap_or_else(|| "canvas".to_string());
            let width = elem.attributes.get("width")
                .and_then(|w| w.parse::<u32>().ok())
                .unwrap_or(300);
            let height = elem.attributes.get("height")
                .and_then(|h| h.parse::<u32>().ok())
                .unwrap_or(150);

            let canvas_data = CanvasData { id, width, height };
            BoxType::CanvasNode(style_node, canvas_data)
        } else {
            match style_node.display().display_type {
                DisplayType::Block => BoxType::BlockNode(style_node),
                DisplayType::Inline => BoxType::InlineNode(style_node),
                DisplayType::Table => BoxType::TableNode(style_node),
                DisplayType::TableRow => BoxType::TableRowNode(style_node),
                DisplayType::TableCell => BoxType::TableCellNode(style_node),
                DisplayType::Flex => BoxType::FlexNode(style_node),
                DisplayType::Grid => BoxType::GridNode(style_node),
                DisplayType::None => panic!("Root node has display: none"),
            }
        }
    } else {
        match style_node.display().display_type {
            DisplayType::Block => BoxType::BlockNode(style_node),
            DisplayType::Inline => BoxType::InlineNode(style_node),
            DisplayType::None => panic!("Root node has display: none"),
            _ => BoxType::InlineNode(style_node), // Text nodes with table display types become inline
        }
    };

    let mut root = LayoutBox::new(box_type);

    for child in &style_node.children {
        match child.display().display_type {
            DisplayType::Block => root.children.push(build_layout_tree(child, image_cache, form_values, input_counter)),
            DisplayType::Inline => root
                .get_inline_container()
                .children
                .push(build_layout_tree(child, image_cache, form_values, input_counter)),
            DisplayType::Table | DisplayType::TableRow | DisplayType::TableCell => {
                root.children.push(build_layout_tree(child, image_cache, form_values, input_counter))
            }
            DisplayType::Flex | DisplayType::Grid => {
                root.children.push(build_layout_tree(child, image_cache, form_values, input_counter))
            }
            DisplayType::None => {}
        }
    }

    root
}

fn create_form_element_data(
    elem: &crate::dom::ElementData,
    form_values: Option<&HashMap<usize, String>>,
    input_counter: &mut usize,
) -> FormElementData {
    let element_type = if elem.tag_name == "button" {
        let button_type = elem.attributes.get("type").map(|s| s.as_str()).unwrap_or("submit");
        match button_type {
            "submit" => FormElementType::Submit,
            _ => FormElementType::Button,
        }
    } else if elem.tag_name == "textarea" {
        FormElementType::Textarea
    } else {
        // input element - check type attribute
        let input_type = elem.attributes.get("type").map(|s| s.as_str()).unwrap_or("text");
        match input_type {
            "password" => FormElementType::Password,
            "email" => FormElementType::Email,
            "number" => FormElementType::Number,
            "checkbox" => FormElementType::Checkbox,
            "radio" => FormElementType::Radio,
            "submit" => FormElementType::Submit,
            "button" => FormElementType::Button,
            _ => FormElementType::TextInput,
        }
    };

    // Determine if this input can be focused and assign index
    let (input_index, current_value) = if matches!(element_type,
        FormElementType::TextInput | FormElementType::Password |
        FormElementType::Email | FormElementType::Number | FormElementType::Textarea) {

        let index = *input_counter;
        *input_counter += 1;

        // Use form_values if available, otherwise use HTML attribute
        let value = form_values
            .and_then(|values| values.get(&index))
            .cloned()
            .or_else(|| elem.attributes.get("value").cloned())
            .unwrap_or_default();

        (Some(index), value)
    } else {
        // Non-focusable inputs (buttons, etc.) just use the value attribute
        (None, elem.attributes.get("value").cloned().unwrap_or_default())
    };

    let placeholder = elem.attributes.get("placeholder").cloned().unwrap_or_default();
    let name = elem.attributes.get("name").cloned().unwrap_or_default();

    FormElementData {
        element_type,
        value: current_value,
        placeholder,
        name,
        input_index,
    }
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
            BoxType::InlineNode(_) | BoxType::AnonymousBlock | BoxType::ImageNode(_, _) | BoxType::FormElement(_, _) | BoxType::CanvasNode(_, _) => self,
            BoxType::BlockNode(_) | BoxType::TableNode(_) | BoxType::TableRowNode(_) | BoxType::TableCellNode(_) | BoxType::FlexNode(_) | BoxType::GridNode(_) => {
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
            BoxType::FormElement(_, _) => self.layout_form_element(containing_block),
            BoxType::CanvasNode(_, _) => self.layout_canvas(containing_block),
            BoxType::TableNode(_) => self.layout_table(containing_block),
            BoxType::TableRowNode(_) => self.layout_table_row(containing_block),
            BoxType::TableCellNode(_) => self.layout_table_cell(containing_block),
            BoxType::FlexNode(_) => self.layout_flex(containing_block),
            BoxType::GridNode(_) => self.layout_grid(containing_block),
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

    fn layout_form_element(&mut self, containing_block: Dimensions) {
        // Set position
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y + containing_block.content.height;

        // Get form element dimensions based on type
        if let BoxType::FormElement(_, form_data) = &self.box_type {
            match form_data.element_type {
                FormElementType::TextInput | FormElementType::Password |
                FormElementType::Email | FormElementType::Number => {
                    // Standard input field dimensions
                    self.dimensions.content.width = 200.0_f32.min(containing_block.content.width);
                    self.dimensions.content.height = 30.0;
                }
                FormElementType::Button | FormElementType::Submit => {
                    // Button dimensions based on content or default
                    let text_width = if form_data.value.is_empty() { 80.0 } else { (form_data.value.len() as f32 * 8.0) + 20.0 };
                    self.dimensions.content.width = text_width.min(containing_block.content.width);
                    self.dimensions.content.height = 32.0;
                }
                FormElementType::Textarea => {
                    // Larger text area
                    self.dimensions.content.width = 300.0_f32.min(containing_block.content.width);
                    self.dimensions.content.height = 100.0;
                }
                FormElementType::Checkbox | FormElementType::Radio => {
                    // Small checkbox/radio button
                    self.dimensions.content.width = 16.0;
                    self.dimensions.content.height = 16.0;
                }
            }
        } else {
            // Default dimensions
            self.dimensions.content.width = 100.0;
            self.dimensions.content.height = 30.0;
        }
    }

    fn layout_canvas(&mut self, containing_block: Dimensions) {
        // Set position
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y + containing_block.content.height;

        // Get canvas dimensions from canvas data
        if let BoxType::CanvasNode(_, canvas_data) = &self.box_type {
            self.dimensions.content.width = canvas_data.width as f32;
            self.dimensions.content.height = canvas_data.height as f32;
        } else {
            // Default canvas dimensions (HTML standard default)
            self.dimensions.content.width = 300.0;
            self.dimensions.content.height = 150.0;
        }
    }

    fn layout_table(&mut self, containing_block: Dimensions) {
        // Simple table layout - treat like a block with auto width
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y + containing_block.content.height;
        self.dimensions.content.width = containing_block.content.width;

        // Layout all table rows
        for child in &mut self.children {
            child.layout(self.dimensions);
            self.dimensions.content.height += child.dimensions.margin_box().height;
        }
    }

    fn layout_table_row(&mut self, containing_block: Dimensions) {
        // Table row spans full table width and arranges cells horizontally
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y + containing_block.content.height;
        self.dimensions.content.width = containing_block.content.width;

        // Count cells to determine width per cell
        let cell_count = self.children.len();
        if cell_count == 0 {
            return;
        }

        let cell_width = containing_block.content.width / cell_count as f32;

        // Layout cells side by side
        let mut x_offset = self.dimensions.content.x;
        let mut max_height = 0.0_f32;

        for child in &mut self.children {
            let mut cell_container = containing_block;
            cell_container.content.x = x_offset;
            cell_container.content.y = self.dimensions.content.y;
            cell_container.content.width = cell_width;
            cell_container.content.height = 0.0;

            child.layout(cell_container);

            x_offset += child.dimensions.margin_box().width;
            max_height = max_height.max(child.dimensions.margin_box().height);
        }

        self.dimensions.content.height = max_height;
    }

    fn layout_table_cell(&mut self, containing_block: Dimensions) {
        // Table cell uses block layout with some padding
        self.dimensions.content.x = containing_block.content.x;
        self.dimensions.content.y = containing_block.content.y;
        self.dimensions.content.width = containing_block.content.width;

        // Add default padding for cells
        self.dimensions.padding.left = 5.0;
        self.dimensions.padding.right = 5.0;
        self.dimensions.padding.top = 5.0;
        self.dimensions.padding.bottom = 5.0;

        // Layout children
        let adjusted_width = containing_block.content.width - 10.0; // Account for padding
        let mut cell_block = containing_block;
        cell_block.content.width = adjusted_width;
        cell_block.content.height = 0.0;

        for child in &mut self.children {
            child.layout(cell_block);
            self.dimensions.content.height += child.dimensions.margin_box().height;
            cell_block.content.height = self.dimensions.content.height;
        }

        // Minimum height for empty cells
        if self.dimensions.content.height < 20.0 {
            self.dimensions.content.height = 20.0;
        }
    }

    fn layout_flex(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        // Get flex properties
        let flex_direction = style.flex_direction();
        let justify_content = style.justify_content();
        let align_items = style.align_items();

        // Set position and width like a block element
        self.calculate_block_width(containing_block);
        self.calculate_block_position(containing_block);

        let is_row = matches!(flex_direction, FlexDirection::Row | FlexDirection::RowReverse);

        // Get available space in main axis
        let available_main = if is_row {
            self.dimensions.content.width
        } else {
            // For column, use containing block height or make it flexible
            containing_block.content.height
        };

        // First pass: layout children to get their natural sizes
        let mut total_flex_grow = 0.0_f32;
        let mut total_base_size = 0.0_f32;

        for child in &mut self.children {
            let child_style = child.get_style_node();
            total_flex_grow += child_style.flex_grow();

            // Do initial layout to get base size
            let mut child_container = self.dimensions;
            child_container.content.height = 0.0;
            child.layout(child_container);

            let base_size = if is_row {
                child.dimensions.margin_box().width
            } else {
                child.dimensions.margin_box().height
            };
            total_base_size += base_size;
        }

        // Calculate remaining space for flex-grow
        let remaining_space = available_main - total_base_size;

        // Second pass: position children with flexbox rules
        let mut main_offset = 0.0_f32;
        let mut max_cross = 0.0_f32;

        // Calculate initial offset for justify-content
        if !is_row || remaining_space <= 0.0 || total_flex_grow > 0.0 {
            // For column or when there's no extra space, start at 0
            main_offset = 0.0;
        } else {
            main_offset = match justify_content {
                JustifyContent::FlexStart => 0.0,
                JustifyContent::FlexEnd => remaining_space,
                JustifyContent::Center => remaining_space / 2.0,
                JustifyContent::SpaceBetween => 0.0,
                JustifyContent::SpaceAround => {
                    if self.children.len() > 0 {
                        remaining_space / (self.children.len() as f32 * 2.0)
                    } else {
                        0.0
                    }
                }
            };
        }

        // Calculate spacing for space-between/space-around
        let spacing = if remaining_space > 0.0 && total_flex_grow == 0.0 {
            match justify_content {
                JustifyContent::SpaceBetween => {
                    if self.children.len() > 1 {
                        remaining_space / (self.children.len() as f32 - 1.0)
                    } else {
                        0.0
                    }
                }
                JustifyContent::SpaceAround => {
                    if self.children.len() > 0 {
                        remaining_space / (self.children.len() as f32)
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            }
        } else {
            0.0
        };

        for child in &mut self.children {
            let child_style = child.get_style_node();
            let flex_grow = child_style.flex_grow();

            // Calculate extra size from flex-grow
            let extra_size = if total_flex_grow > 0.0 && remaining_space > 0.0 {
                (flex_grow / total_flex_grow) * remaining_space
            } else {
                0.0
            };

            if is_row {
                // Row direction: main axis is horizontal
                child.dimensions.content.x = self.dimensions.content.x + main_offset;
                child.dimensions.content.y = self.dimensions.content.y;

                // Apply flex-grow to width
                if extra_size > 0.0 {
                    child.dimensions.content.width += extra_size;
                }

                // Handle align-items (cross axis)
                let child_height = child.dimensions.margin_box().height;
                match align_items {
                    AlignItems::FlexStart => {
                        child.dimensions.content.y = self.dimensions.content.y;
                    }
                    AlignItems::FlexEnd => {
                        child.dimensions.content.y = self.dimensions.content.y +
                            (self.dimensions.content.height - child_height);
                    }
                    AlignItems::Center => {
                        child.dimensions.content.y = self.dimensions.content.y +
                            (self.dimensions.content.height - child_height) / 2.0;
                    }
                    AlignItems::Stretch => {
                        // Could stretch height here
                    }
                }

                main_offset += child.dimensions.margin_box().width;
                max_cross = max_cross.max(child_height);

                // Add spacing
                if matches!(justify_content, JustifyContent::SpaceBetween | JustifyContent::SpaceAround) {
                    main_offset += spacing;
                }
            } else {
                // Column direction: main axis is vertical
                child.dimensions.content.x = self.dimensions.content.x;
                child.dimensions.content.y = self.dimensions.content.y + main_offset;

                // Apply flex-grow to height
                if extra_size > 0.0 {
                    child.dimensions.content.height += extra_size;
                }

                // Handle align-items (cross axis)
                let child_width = child.dimensions.margin_box().width;
                match align_items {
                    AlignItems::FlexStart => {
                        child.dimensions.content.x = self.dimensions.content.x;
                    }
                    AlignItems::FlexEnd => {
                        child.dimensions.content.x = self.dimensions.content.x +
                            (self.dimensions.content.width - child_width);
                    }
                    AlignItems::Center => {
                        child.dimensions.content.x = self.dimensions.content.x +
                            (self.dimensions.content.width - child_width) / 2.0;
                    }
                    AlignItems::Stretch => {
                        // Could stretch width here
                    }
                }

                main_offset += child.dimensions.margin_box().height;
                max_cross = max_cross.max(child_width);

                // Add spacing
                if matches!(justify_content, JustifyContent::SpaceBetween | JustifyContent::SpaceAround) {
                    main_offset += spacing;
                }
            }
        }

        // Handle reverse directions
        if matches!(flex_direction, FlexDirection::RowReverse | FlexDirection::ColumnReverse) {
            // Reverse the positions of children
            if is_row {
                for child in &mut self.children {
                    let distance_from_start = child.dimensions.content.x - self.dimensions.content.x;
                    child.dimensions.content.x = self.dimensions.content.x +
                        (self.dimensions.content.width - distance_from_start - child.dimensions.content.width);
                }
            } else {
                for child in &mut self.children {
                    let distance_from_start = child.dimensions.content.y - self.dimensions.content.y;
                    child.dimensions.content.y = self.dimensions.content.y +
                        (self.dimensions.content.height - distance_from_start - child.dimensions.content.height);
                }
            }
        }

        // Set container's content height
        if is_row {
            self.dimensions.content.height = max_cross;
        } else {
            self.dimensions.content.height = main_offset;
        }
    }

    fn layout_grid(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        // Get grid properties
        let column_tracks = style.grid_template_columns();
        let row_tracks = style.grid_template_rows();
        let gap = style.grid_gap();

        // Set position and width like a block element
        self.calculate_block_width(containing_block);
        self.calculate_block_position(containing_block);

        // If no grid template is defined, fall back to auto-flow
        if column_tracks.is_empty() && row_tracks.is_empty() {
            // Auto-flow: arrange children in a simple grid
            let num_children = self.children.len();
            if num_children == 0 {
                return;
            }

            // Default to 2 columns for auto-flow
            let num_cols = 2;
            let num_rows = (num_children + num_cols - 1) / num_cols;

            let col_width = (self.dimensions.content.width - gap * (num_cols as f32 - 1.0)) / num_cols as f32;

            let mut row = 0;
            let mut col = 0;
            let mut max_row_height = 0.0_f32;
            let mut y_offset = self.dimensions.content.y;

            for child in &mut self.children {
                // Calculate position
                let x = self.dimensions.content.x + (col as f32 * (col_width + gap));
                let y = y_offset;

                // Layout child
                let mut child_container = self.dimensions;
                child_container.content.x = x;
                child_container.content.y = y;
                child_container.content.width = col_width;
                child_container.content.height = 0.0;

                child.layout(child_container);

                max_row_height = max_row_height.max(child.dimensions.margin_box().height);

                col += 1;
                if col >= num_cols {
                    col = 0;
                    row += 1;
                    y_offset += max_row_height + gap;
                    max_row_height = 0.0;
                }
            }

            self.dimensions.content.height = y_offset - self.dimensions.content.y;
            return;
        }

        // Calculate column sizes
        let num_cols = if !column_tracks.is_empty() {
            column_tracks.len()
        } else {
            1
        };

        let num_rows = if !row_tracks.is_empty() {
            row_tracks.len()
        } else {
            (self.children.len() + num_cols - 1) / num_cols
        };

        // Calculate available space
        let available_width = self.dimensions.content.width - gap * ((num_cols - 1) as f32);
        let available_height = containing_block.content.height;

        // Resolve column track sizes
        let col_sizes = resolve_track_sizes(&column_tracks, available_width);
        let row_sizes = if !row_tracks.is_empty() {
            resolve_track_sizes(&row_tracks, available_height)
        } else {
            vec![100.0; num_rows] // Default row height
        };

        // Position children in grid
        let mut child_idx = 0;
        for row_idx in 0..num_rows {
            for col_idx in 0..num_cols {
                if child_idx >= self.children.len() {
                    break;
                }

                let child = &mut self.children[child_idx];

                // Calculate position
                let x = self.dimensions.content.x +
                    col_sizes[0..col_idx].iter().sum::<f32>() +
                    (col_idx as f32 * gap);

                let y = self.dimensions.content.y +
                    row_sizes[0..row_idx].iter().sum::<f32>() +
                    (row_idx as f32 * gap);

                // Set child dimensions
                child.dimensions.content.x = x;
                child.dimensions.content.y = y;
                child.dimensions.content.width = col_sizes[col_idx];
                child.dimensions.content.height = row_sizes[row_idx];

                // Layout child with these constraints
                let mut child_container = self.dimensions;
                child_container.content.x = x;
                child_container.content.y = y;
                child_container.content.width = col_sizes[col_idx];
                child_container.content.height = row_sizes[row_idx];

                child.layout(child_container);

                child_idx += 1;
            }
        }

        // Calculate total grid height
        let total_height = row_sizes.iter().sum::<f32>() + (gap * (num_rows - 1) as f32);
        self.dimensions.content.height = total_height;
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
            BoxType::BlockNode(node) | BoxType::InlineNode(node) | BoxType::ImageNode(node, _) |
            BoxType::FormElement(node, _) | BoxType::CanvasNode(node, _) | BoxType::TableNode(node) | BoxType::TableRowNode(node) |
            BoxType::TableCellNode(node) | BoxType::FlexNode(node) | BoxType::GridNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block has no style node"),
        }
    }

    fn apply_positioning(&mut self) {
        // Apply positioning to this box
        if let Some(style) = self.try_get_style_node() {
            let position = style.position();

            match position {
                Position::Static => {
                    // No positioning adjustments
                }
                Position::Relative => {
                    // Offset from normal position
                    if let Some(top) = style.top() {
                        self.dimensions.content.y += top;
                    }
                    if let Some(left) = style.left() {
                        self.dimensions.content.x += left;
                    }
                    // Note: bottom and right would require calculating from container
                }
                Position::Absolute | Position::Fixed => {
                    // For absolute/fixed, override position entirely
                    // Note: This is a simplified implementation
                    // Full implementation would find containing block or viewport

                    if let Some(top) = style.top() {
                        self.dimensions.content.y = top;
                    }
                    if let Some(left) = style.left() {
                        self.dimensions.content.x = left;
                    }
                }
            }
        }

        // Recursively apply to children
        for child in &mut self.children {
            child.apply_positioning();
        }
    }

    fn try_get_style_node(&self) -> Option<&'a StyledNode<'a>> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) | BoxType::ImageNode(node, _) |
            BoxType::FormElement(node, _) | BoxType::CanvasNode(node, _) | BoxType::TableNode(node) | BoxType::TableRowNode(node) |
            BoxType::TableCellNode(node) | BoxType::FlexNode(node) | BoxType::GridNode(node) => Some(node),
            BoxType::AnonymousBlock => None,
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

// Helper function to resolve grid track sizes
fn resolve_track_sizes(tracks: &[GridTrackSize], available_space: f32) -> Vec<f32> {
    if tracks.is_empty() {
        return vec![];
    }

    let mut sizes = vec![0.0; tracks.len()];
    let mut total_fr = 0.0;
    let mut used_space = 0.0;

    // First pass: resolve fixed (px) and auto sizes
    for (i, track) in tracks.iter().enumerate() {
        match track {
            GridTrackSize::Px(px) => {
                sizes[i] = *px;
                used_space += px;
            }
            GridTrackSize::Fr(fr) => {
                total_fr += fr;
            }
            GridTrackSize::Auto => {
                // For auto, use a default size for now
                sizes[i] = 100.0;
                used_space += 100.0;
            }
        }
    }

    // Second pass: distribute remaining space to fr units
    if total_fr > 0.0 {
        let remaining_space = (available_space - used_space).max(0.0);
        let fr_size = remaining_space / total_fr;

        for (i, track) in tracks.iter().enumerate() {
            if let GridTrackSize::Fr(fr) = track {
                sizes[i] = fr * fr_size;
            }
        }
    }

    sizes
}
