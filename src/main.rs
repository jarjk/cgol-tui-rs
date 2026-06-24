use std::io::Write as _;
use std::{io::Read, str::FromStr};

use app::{App, Universe};
use ratatui::backend::TerminaBackend;
use ratatui::termina::escape::csi::{self, Csi};
use ratatui::termina::{EventReader, PlatformTerminal, Terminal as _};
use ratatui::Terminal;

pub mod app;

type AppTerminal = Terminal<TerminaBackend<PlatformTerminal>>;

macro_rules! decset {
    ($mode:ident) => {{
        let mode = csi::DecPrivateMode::Code(csi::DecPrivateModeCode::$mode);
        Csi::Mode(csi::Mode::SetDecPrivateMode(mode))
    }};
}

macro_rules! decreset {
    ($mode:ident) => {{
        let mode = csi::DecPrivateMode::Code(csi::DecPrivateModeCode::$mode);
        Csi::Mode(csi::Mode::ResetDecPrivateMode(mode))
    }};
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg_universes = parse_args()?;

    let (mut terminal, events) = init_terminal()?;
    let mut app = App::default().with_universes(arg_universes);
    let res = app.run(&mut terminal, &events);

    restore_terminal(&mut terminal)?;

    Ok(res?)
}

fn init_terminal() -> Result<(AppTerminal, EventReader), Box<dyn std::error::Error>> {
    let mut output = PlatformTerminal::new()?;
    output.enter_raw_mode()?;

    let enter_alternate_screen = decset!(ClearAndEnableAlternateScreen);
    let show_cursor = decset!(ShowCursor);
    write!(output, "{enter_alternate_screen}{show_cursor}")?;
    output.flush()?;

    let events = output.event_reader();
    let backend = TerminaBackend::new(output);
    let terminal = Terminal::new(backend)?;
    Ok((terminal, events))
}

fn restore_terminal(terminal: &mut AppTerminal) -> Result<(), Box<dyn std::error::Error>> {
    let leave_alternate_screen = decreset!(ClearAndEnableAlternateScreen);
    let show_cursor = decset!(ShowCursor);
    let backend = terminal.backend_mut();
    write!(backend, "{leave_alternate_screen}{show_cursor}")?;
    backend.flush()?;
    Ok(())
}

fn parse_args() -> Result<Vec<Universe>, Box<dyn std::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.contains(&"-h".into()) || args.contains(&"--help".into()) {
        println!(
            "A Conway's Game of Life viewer TUI.
            
USAGE: cgol-tui [<pattern>,...]

where <pattern> is either a .cells file, or - for stdin"
        );
        std::process::exit(0);
    }
    let piped_universe = {
        let mut univ = String::new();
        if args.len() == 1 && args[0] == "-" {
            std::io::stdin().read_to_string(&mut univ)?;
        }
        if univ.is_empty() {
            vec![]
        } else {
            vec![Universe::from_str(&univ)?]
        }
    };
    let universes = args
        .iter()
        .flat_map(std::fs::read_to_string)
        .flat_map(|s| Universe::from_str(&s))
        .collect::<Vec<_>>();

    Ok([universes, piped_universe].concat())
}
