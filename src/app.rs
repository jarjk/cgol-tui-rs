pub use area::Area;
pub use cell::Cell;
use ratatui::{backend::Backend, Terminal};
use ratatui_manoterm::Key;
use std::{io, str::FromStr, time::Duration};
pub use universe::Universe;

/// Default poll duration
const DEF_DUR: Duration = Duration::from_millis(400);
/// Pause duration: a day
const PAUSE: Duration = Duration::from_secs(60 * 60 * 24);

mod area;
mod cell;
/// Starting shapes
pub mod shapes;
/// ui
mod ui;
/// Conway's Game of Life universe
mod universe;

#[cfg(test)]
mod tests;

pub struct App {
    pub available_universes: Vec<Universe>,
    universe: Universe,
    i: usize,
    pub poll_t: Duration,
    pub area: Area,
}
impl Default for App {
    fn default() -> Self {
        App {
            area: Area::default(),
            universe: Universe::default(),
            i: 0,
            poll_t: DEF_DUR,
            available_universes: shapes::all(),
        }
    }
}
impl App {
    pub fn with_universes(self, universes: Vec<Universe>) -> Self {
        Self {
            available_universes: [universes, shapes::all()].concat(),
            ..self
        }
    }
    pub fn new(area: Area, available_universes: Vec<Universe>, poll_t: Duration) -> Self {
        App {
            area,
            universe: available_universes[0].clone(),
            i: 0,
            poll_t,
            available_universes,
        }
    }
    pub fn paused(&self) -> bool {
        self.poll_t == PAUSE
    }
    pub fn len(&self) -> usize {
        self.available_universes.len() + shapes::N
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn get(&self) -> Universe {
        let true_len = self.available_universes.len();
        if self.i < true_len {
            self.available_universes.get(self.i).unwrap().clone()
        } else {
            shapes::get_special(self.i - true_len, self.area)
        }
    }

    // pub fn render_universe(&self) {
    //     println!("{}", self.universe);
    // }

    pub fn play_pause(&mut self, prev_poll_t: &mut Duration) {
        if self.paused() {
            self.poll_t = *prev_poll_t;
        } else {
            *prev_poll_t = self.poll_t;
            self.poll_t = PAUSE;
        }
    }
    pub fn restart(&mut self) {
        let figur = self.get();
        self.universe = Universe::from_figur(self.area, figur)
            .expect("display area should be big enough to fit this figure");
    }

    pub fn tick(&mut self) {
        self.universe.tick();
    }

    pub fn faster(&mut self, big: bool) {
        if !self.paused() {
            let div = if big { 2 } else { 5 };
            self.poll_t = self
                .poll_t
                .checked_sub(self.poll_t.checked_div(div).unwrap_or(DEF_DUR))
                .unwrap_or(DEF_DUR);
        }
    }
    pub fn slower(&mut self, big: bool) {
        if !self.paused() {
            let div = if big { 2 } else { 5 };
            self.poll_t = self
                .poll_t
                .checked_add(self.poll_t.checked_div(div).unwrap_or(DEF_DUR))
                .unwrap_or(DEF_DUR);
        }
    }

    pub fn next(&mut self) {
        if self.i + 1 == self.len() {
            self.i = 0;
        } else {
            self.i += 1;
        }
        self.restart();
    }
    pub fn prev(&mut self) {
        if self.i > 0 {
            self.i -= 1;
        } else {
            self.i = self.len() - 1;
        }
        self.restart();
    }
    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let mut prev_poll_t = self.poll_t;
        let k_recv = Key::receiver();

        loop {
            terminal
                .draw(|f| ui::ui(f, self))
                .map_err(|e| io::Error::other(format!("failed to draw frame: {e}")))?;
            let read_key = k_recv.recv_timeout(self.poll_t).unwrap_or_default();

            // Wait up to `poll_t` for another event
            if let Some(key) = read_key {
                match key {
                    Key::Char('q') | Key::Escape => break,
                    Key::Char('j') | Key::Down => self.slower(false),
                    Key::Char('k') | Key::Up => self.faster(false),
                    Key::Char(' ' | '\n') => self.play_pause(&mut prev_poll_t),
                    Key::Char('r') => self.restart(),
                    Key::Char('n' | 'l') | Key::Right => self.next(),
                    Key::Char('p' | 'h') | Key::Left => self.prev(),
                    Key::Char('R') | Key::Backspace => *self = Self::default(),
                    _ => {}
                }
            } else {
                // Timeout expired, updating life state
                self.tick();
            }
        }

        Ok(())
    }
}
