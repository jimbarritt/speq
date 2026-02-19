use crate::spec::LoadedSpec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pane {
    SchemaList,
    Detail,
}

pub struct App {
    pub spec: LoadedSpec,
    pub selected: usize,
    pub focused_pane: Pane,
    pub should_quit: bool,
}

impl App {
    pub fn new(spec: LoadedSpec) -> Self {
        App {
            spec,
            selected: 0,
            focused_pane: Pane::SchemaList,
            should_quit: false,
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let max = self.spec.schema_names.len().saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
        }
    }

    pub fn goto_top(&mut self) {
        self.selected = 0;
    }

    pub fn goto_bottom(&mut self) {
        self.selected = self.spec.schema_names.len().saturating_sub(1);
    }

    pub fn toggle_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            Pane::SchemaList => Pane::Detail,
            Pane::Detail => Pane::SchemaList,
        };
    }

    pub fn selected_schema_name(&self) -> Option<&str> {
        self.spec.schema_names.get(self.selected).map(|s| s.as_str())
    }
}
