use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyEventKind},
};
use std::{error::Error, io, time::Duration};

mod app;
mod borg;
mod browser;
mod checker;
mod config;
mod events;
mod i18n;
mod ui;

use app::App;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    let mut app = App::new();
    app.on_start();

    let res = run_app(&mut terminal, &mut app);

    ratatui::restore();

    if let Err(err) = res {
        println!("Erro na aplicação: {err:?}");
    }

    Ok(())
}

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        app.handle_background_tasks();

        if event::poll(Duration::from_millis(40))?
            && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press {
                    events::handle_key_event(app, key);
                }

        if app.should_quit {
            return Ok(());
        }
    }
}
