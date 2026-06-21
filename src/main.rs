//! Main entrypoint: terminal lifecycle, run loop, UI draw, and delegation.

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
};
use std::{error::Error, io};

mod actions;
mod app;
mod capture_nl;
mod config;
mod date_input;
mod editor;
mod export;
mod input;
mod integrations;
mod links;
mod models;
mod runtime;
mod saved_views;
mod storage;
mod task_metadata;
mod ui;

use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;

    // Keyboard enhancement flags may fail on unsupported terminals (e.g., Windows Legacy Console).
    // Errors are ignored as they don't affect app functionality.
    let _ = execute!(
        stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    );

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();
    app.init_image_picker();

    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    let _ = execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags);

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let poll_interval = std::time::Duration::from_millis(app.config.ui.poll_interval_ms);

    loop {
        runtime::tick(app);

        terminal.draw(|f| ui::ui(f, app))?;

        // Block normal input during the pomodoro completion alert, but let any key press
        // dismiss it early ("press any key to continue"). The alert also auto-expires, so
        // no interaction is required; consuming the key here is the acknowledgement.
        if app.pomodoro_alert_expiry.is_some() {
            if event::poll(std::time::Duration::from_millis(100))?
                && let crossterm::event::Event::Key(_) = event::read()?
            {
                app.pomodoro_alert_expiry = None;
                app.pomodoro_alert_message = None;
            }
            continue;
        }

        if event::poll(poll_interval)? {
            let event = event::read()?;
            input::handle_event(app, event);
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
