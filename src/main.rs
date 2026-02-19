mod app;
mod parser;
mod spec;
mod ui;

use std::{env, fs, io};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;

fn main() -> Result<()> {
    // Determine the spec path: first CLI arg, or the built-in fixture for dev.
    let spec_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "fixtures/petstore.yaml".to_string());

    let content = fs::read_to_string(&spec_path)
        .with_context(|| format!("cannot read spec file: {spec_path}"))?;

    let spec = parser::parse_spec(&content)
        .with_context(|| format!("failed to parse spec: {spec_path}"))?;

    // Set up the terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(spec);

    // Run the event loop; restore terminal afterwards even on error
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                handle_key(app, key.code, key.modifiers);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    match (code, modifiers) {
        // Quit
        (KeyCode::Char('q'), KeyModifiers::NONE) => app.should_quit = true,
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => app.should_quit = true,

        // Navigation
        (KeyCode::Char('j'), KeyModifiers::NONE) | (KeyCode::Down, KeyModifiers::NONE) => {
            app.move_down()
        }
        (KeyCode::Char('k'), KeyModifiers::NONE) | (KeyCode::Up, KeyModifiers::NONE) => {
            app.move_up()
        }
        (KeyCode::Char('g'), KeyModifiers::NONE) => app.goto_top(),
        (KeyCode::Char('G'), KeyModifiers::SHIFT) => app.goto_bottom(),

        // Pane switching
        (KeyCode::Tab, KeyModifiers::NONE) => app.toggle_pane(),

        _ => {}
    }
}
