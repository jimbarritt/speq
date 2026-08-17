mod app;
mod parser;
mod spec;
mod tree;
mod ui;

use std::{fs, io, path::PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;

const DEV_FIXTURE: &str = "fixtures/petstore.yaml";

#[derive(Parser)]
#[command(
    name = "speq",
    version,
    about = "OpenAPI specification browser",
    long_about = "speq — a keyboard-driven, read-only browser for OpenAPI and Swagger specifications."
)]
struct Cli {
    #[arg(
        value_name = "SPEC",
        help = "Path to an OpenAPI or Swagger specification file (YAML or JSON)"
    )]
    spec: Option<PathBuf>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let spec_path = match cli.spec {
        Some(path) => path,
        None => {
            let fixture = PathBuf::from(DEV_FIXTURE);
            if !fixture.is_file() {
                bail!(
                    "no spec file given\n\nUsage: speq <SPEC>\n\nFor more information, try '--help'."
                );
            }
            fixture
        }
    };

    let display = spec_path.display();

    let content = fs::read_to_string(&spec_path)
        .with_context(|| format!("cannot read spec file: {display}"))?;

    let spec = parser::parse_spec(&content)
        .with_context(|| format!("failed to parse spec: {display}"))?;

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

        if event::poll(std::time::Duration::from_millis(16))?
            && let Event::Key(key) = event::read()?
        {
            handle_key(app, key.code, key.modifiers);
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_key(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
    // Handle pending two-key sequences first.
    if let Some(pending) = app.pending_key.take() {
        match (pending, code, modifiers) {
            // gg → goto top
            ('g', KeyCode::Char('g'), KeyModifiers::NONE) => {
                app.goto_top();
                return;
            }
            // zo → expand node
            ('z', KeyCode::Char('o'), KeyModifiers::NONE) => {
                app.expand_node();
                return;
            }
            // zc → collapse node
            ('z', KeyCode::Char('c'), KeyModifiers::NONE) => {
                app.collapse_node();
                return;
            }
            // zR → expand all
            ('z', KeyCode::Char('R'), KeyModifiers::SHIFT) => {
                app.expand_all();
                return;
            }
            // zM → collapse all
            ('z', KeyCode::Char('M'), KeyModifiers::SHIFT) => {
                app.collapse_all();
                return;
            }
            // Unrecognised second key — fall through to normal handling below.
            _ => {}
        }
    }

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
        // First key of 'gg' sequence
        (KeyCode::Char('g'), KeyModifiers::NONE) => app.pending_key = Some('g'),
        (KeyCode::Char('G'), KeyModifiers::SHIFT) => app.goto_bottom(),

        // Expand / collapse
        (KeyCode::Char('l'), KeyModifiers::NONE) => app.toggle_expand(),
        (KeyCode::Char('h'), KeyModifiers::NONE) => app.collapse_node(),

        // Two-key 'z' sequences
        (KeyCode::Char('z'), KeyModifiers::NONE) => app.pending_key = Some('z'),

        // Pane switching
        (KeyCode::Tab, KeyModifiers::NONE) => app.toggle_pane(),

        // Detail pane scrolling
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => app.scroll_detail_down(),
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => app.scroll_detail_up(),

        _ => {}
    }
}
