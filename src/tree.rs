/// Kind of schema node.
#[derive(Debug, Clone)]
pub enum NodeKind {
    Schema,      // top-level schema root (no specific type)
    Object,
    Array,
    Str,         // string (avoid shadowing std::string::String)
    Integer,
    Number,
    Boolean,
    Ref(String), // $ref — contains the resolved target schema name
    AllOf,
    OneOf,
    AnyOf,
    Unknown,
}

/// Metadata for a tree node.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub kind: NodeKind,
    pub format: Option<String>,
    pub description: Option<String>,
    pub required: bool,
    pub constraints: Vec<String>,    // e.g. "min: 0", "maxLength: 255"
    pub enum_values: Vec<String>,    // formatted enum variants
    pub example: Option<String>,     // JSON-formatted
    pub default_val: Option<String>, // JSON-formatted
}

/// A node in the schema tree.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub info: NodeInfo,
    pub children: Vec<TreeNode>,
    pub expanded: bool,
}

impl TreeNode {
    pub fn is_expandable(&self) -> bool {
        !self.children.is_empty()
    }

    pub fn type_label(&self) -> String {
        match &self.info.kind {
            NodeKind::Schema  => "schema".to_string(),
            NodeKind::Object  => "object".to_string(),
            NodeKind::Array   => "array".to_string(),
            NodeKind::Str     => "string".to_string(),
            NodeKind::Integer => "integer".to_string(),
            NodeKind::Number  => "number".to_string(),
            NodeKind::Boolean => "boolean".to_string(),
            NodeKind::Ref(t)  => format!("→{}", t),
            NodeKind::AllOf   => "allOf".to_string(),
            NodeKind::OneOf   => "oneOf".to_string(),
            NodeKind::AnyOf   => "anyOf".to_string(),
            NodeKind::Unknown => "?".to_string(),
        }
    }
}

/// A node in the flattened visible list (borrowed from the tree).
pub struct FlatNode<'a> {
    pub node: &'a TreeNode,
    pub depth: usize,
}

/// Tree state: all schema roots + cursor into the flat visible list.
pub struct TreeState {
    pub roots: Vec<TreeNode>,
    pub cursor: usize,
}

impl TreeState {
    pub fn new(roots: Vec<TreeNode>) -> Self {
        TreeState { roots, cursor: 0 }
    }

    /// Flatten visible nodes depth-first, skipping collapsed subtrees.
    pub fn flatten(&self) -> Vec<FlatNode<'_>> {
        let mut result = Vec::new();
        for root in &self.roots {
            flatten_node(root, 0, &mut result);
        }
        result
    }

    pub fn visible_count(&self) -> usize {
        self.flatten().len()
    }

    pub fn selected_node(&self) -> Option<&TreeNode> {
        self.flatten().into_iter().nth(self.cursor).map(|f| f.node)
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let max = self.visible_count().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
        }
    }

    pub fn goto_top(&mut self) {
        self.cursor = 0;
    }

    pub fn goto_bottom(&mut self) {
        self.cursor = self.visible_count().saturating_sub(1);
    }

    /// Toggle expand/collapse on the node at the cursor.
    pub fn toggle_at_cursor(&mut self) {
        let target = self.cursor;
        let mut counter = 0;
        toggle_at(&mut self.roots, target, &mut counter);
    }

    /// Expand (only) the node at the cursor.
    pub fn expand_at_cursor(&mut self) {
        let target = self.cursor;
        let mut counter = 0;
        set_expanded_at(&mut self.roots, target, &mut counter, true);
    }

    /// Collapse (only) the node at the cursor.
    pub fn collapse_at_cursor(&mut self) {
        let target = self.cursor;
        let mut counter = 0;
        set_expanded_at(&mut self.roots, target, &mut counter, false);
    }

    /// Expand every node in the entire tree.
    pub fn expand_all(&mut self) {
        set_expanded_all(&mut self.roots, true);
    }

    /// Collapse every node and reset cursor to top.
    pub fn collapse_all(&mut self) {
        set_expanded_all(&mut self.roots, false);
        self.cursor = 0;
    }
}

// ── internal helpers ──────────────────────────────────────────────────────────

fn flatten_node<'a>(node: &'a TreeNode, depth: usize, out: &mut Vec<FlatNode<'a>>) {
    out.push(FlatNode { node, depth });
    if node.expanded {
        for child in &node.children {
            flatten_node(child, depth + 1, out);
        }
    }
}

/// Walk visible nodes counting as we go; toggle the node whose flat index == target.
fn toggle_at(nodes: &mut Vec<TreeNode>, target: usize, counter: &mut usize) -> bool {
    for node in nodes.iter_mut() {
        if *counter == target {
            if node.is_expandable() {
                node.expanded = !node.expanded;
            }
            return true;
        }
        *counter += 1;
        if node.expanded && toggle_at(&mut node.children, target, counter) {
            return true;
        }
    }
    false
}

fn set_expanded_at(
    nodes: &mut Vec<TreeNode>,
    target: usize,
    counter: &mut usize,
    expanded: bool,
) -> bool {
    for node in nodes.iter_mut() {
        if *counter == target {
            if node.is_expandable() {
                node.expanded = expanded;
            }
            return true;
        }
        *counter += 1;
        if node.expanded && set_expanded_at(&mut node.children, target, counter, expanded) {
            return true;
        }
    }
    false
}

fn set_expanded_all(nodes: &mut Vec<TreeNode>, expanded: bool) {
    for node in nodes.iter_mut() {
        if node.is_expandable() {
            node.expanded = expanded;
        }
        set_expanded_all(&mut node.children, expanded);
    }
}
