use crate::spec::LoadedSpec;
use crate::tree::TreeState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pane {
    SchemaList,
    Detail,
}

pub struct App {
    pub spec: LoadedSpec,
    pub tree: TreeState,
    pub focused_pane: Pane,
    pub should_quit: bool,
    pub detail_scroll: u16,
    pub pending_key: Option<char>,
}

impl App {
    pub fn new(spec: LoadedSpec) -> Self {
        let tree = TreeState::new(spec.schema_nodes.clone());
        App {
            spec,
            tree,
            focused_pane: Pane::SchemaList,
            should_quit: false,
            detail_scroll: 0,
            pending_key: None,
        }
    }

    pub fn move_up(&mut self) {
        self.tree.move_up();
        self.detail_scroll = 0;
    }

    pub fn move_down(&mut self) {
        self.tree.move_down();
        self.detail_scroll = 0;
    }

    pub fn goto_top(&mut self) {
        self.tree.goto_top();
        self.detail_scroll = 0;
    }

    pub fn goto_bottom(&mut self) {
        self.tree.goto_bottom();
        self.detail_scroll = 0;
    }

    pub fn toggle_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            Pane::SchemaList => Pane::Detail,
            Pane::Detail => Pane::SchemaList,
        };
    }

    pub fn toggle_expand(&mut self) {
        self.tree.toggle_at_cursor();
        self.detail_scroll = 0;
    }

    pub fn expand_node(&mut self) {
        self.tree.expand_at_cursor();
    }

    pub fn collapse_node(&mut self) {
        self.tree.collapse_at_cursor();
    }

    pub fn expand_all(&mut self) {
        self.tree.expand_all();
    }

    pub fn collapse_all(&mut self) {
        self.tree.collapse_all();
        self.detail_scroll = 0;
    }

    pub fn scroll_detail_down(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_add(3);
    }

    pub fn scroll_detail_up(&mut self) {
        self.detail_scroll = self.detail_scroll.saturating_sub(3);
    }
}
