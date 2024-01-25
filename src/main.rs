pub mod app;
pub mod event;
pub mod game_ui;
pub mod menu_ui;
pub mod tui;
pub mod update;

use app::App;
use clap::Parser;
use color_eyre::Result;
use event::{Event, EventHandler};
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::Tui;
use update::update;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'H', long)]
    height: Option<u8>,
    #[arg(short, long)]
    width: Option<u8>,
    #[arg(short, long)]
    mines: Option<u16>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Create the terminal application.
    let mut app = App::new(args.height, args.width, args.mines)
        .expect("Couldn't create the app instance. Bad parameters?");

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.enter()?;

    // Start the main loop.
    while !app.should_quit {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => update(&mut app, key_event).unwrap(),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        };
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
