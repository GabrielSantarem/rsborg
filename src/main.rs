use ratatui::{
    DefaultTerminal,
    crossterm::event::{self, Event, KeyEventKind},
};
use std::{error::Error, io, process, time::Duration};

mod app;
mod borg;
mod browser;
mod checker;
mod config;
mod events;
mod i18n;
pub mod logger;
mod ui;

use app::App;

struct CliArgs {
    repo_path: Option<String>,
    lang: Option<String>,
}

fn print_help() {
    println!(
        "\x1b[1;36mRsBorg\x1b[0m \x1b[1mv{}\x1b[0m - Interface de Terminal para BorgBackup\n\n\
        \x1b[1mUSO:\x1b[0m\n    \
        rsborg [OPÇÕES] [CAMINHO_DO_REPOSITÓRIO]\n\n\
        \x1b[1mARGUMENTOS:\x1b[0m\n    \
        \x1b[32m<CAMINHO_DO_REPOSITÓRIO>\x1b[0m   Caminho opcional do repositório Borg para abrir diretamente\n\n\
        \x1b[1mOPÇÕES:\x1b[0m\n    \
        \x1b[32m-r, --repo <CAMINHO>\x1b[0m       Especifica o caminho do repositório Borg\n    \
        \x1b[32m-l, --lang <pt|en>\x1b[0m         Força o idioma inicial (Português ou Inglês)\n    \
        \x1b[32m-V, -v, --version\x1b[0m          Exibe a versão do programa\n    \
        \x1b[32m-h, --help\x1b[0m                 Exibe este menu de ajuda e opções\n",
        env!("CARGO_PKG_VERSION")
    );
}

fn parse_cli_args() -> Result<Option<CliArgs>, ()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut repo_path = None;
    let mut lang = None;
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return Ok(None);
            }
            "-V" | "-v" | "--version" => {
                println!("rsborg {}", env!("CARGO_PKG_VERSION"));
                return Ok(None);
            }
            "-r" | "--repo" => {
                if i + 1 < args.len() {
                    repo_path = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("\x1b[31mErro:\x1b[0m A opção '--repo' requer um caminho de repositório.");
                    return Err(());
                }
            }
            "-l" | "--lang" => {
                if i + 1 < args.len() {
                    lang = Some(args[i + 1].clone());
                    i += 1;
                } else {
                    eprintln!("\x1b[31mErro:\x1b[0m A opção '--lang' requer 'pt' ou 'en'.");
                    return Err(());
                }
            }
            arg if !arg.starts_with('-') && repo_path.is_none() => {
                repo_path = Some(arg.to_string());
            }
            unknown => {
                eprintln!(
                    "\x1b[31mErro:\x1b[0m Argumento desconhecido '{}'. Use \x1b[1m--help\x1b[0m para ver as opções.",
                    unknown
                );
                return Err(());
            }
        }
        i += 1;
    }

    Ok(Some(CliArgs { repo_path, lang }))
}

fn setup_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();
        log_error!("FATAL PANIC: {:?}", panic_info);
        eprintln!("\n\x1b[31;1m[RsBorg Fatal Error]\x1b[0m Ocorreu um erro inesperado no aplicativo:");
        original_hook(panic_info);
        eprintln!("\x1b[33mO terminal foi restaurado com segurança para o modo padrão.\x1b[0m\n");
    }));
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = match parse_cli_args() {
        Ok(Some(cli)) => cli,
        Ok(None) => return Ok(()),
        Err(()) => process::exit(1),
    };

    setup_panic_hook();
    logger::init_logger();
    log_info!("Application starting up with CLI arguments: repo={:?}, lang={:?}", cli.repo_path, cli.lang);

    let mut terminal = ratatui::init();

    let mut app = App::new();
    if let Some(ref lang) = cli.lang {
        app.set_language(lang);
    }
    if let Some(ref path) = cli.repo_path {
        app.override_repo_path(path);
    }
    app.on_start();

    let res = run_app(&mut terminal, &mut app);

    ratatui::restore();

    if let Err(ref err) = res {
        log_error!("Interface execution error: {err:?}");
        eprintln!("Erro na execução da interface: {err:?}");
    } else {
        log_info!("Application exited cleanly.");
    }

    Ok(())
}

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        app.handle_background_tasks();

        if event::poll(Duration::from_millis(40))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            events::handle_key_event(app, key);
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
