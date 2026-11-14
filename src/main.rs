mod game;
mod render;
mod snake;

use std::fs;
use std::io::{self, Stdout, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{cursor, execute};
use rand::rngs::StdRng;
use rand::SeedableRng;

use game::{Board, Game, GameState};
use snake::Direction;

/// Largest board we'll use; smaller terminals get a board that fits.
const MAX_BOARD: Board = Board { width: 30, height: 20 };
/// Below this the game isn't really playable; show "too small" instead.
const MIN_BOARD: Board = Board { width: 10, height: 8 };

/// Picks the biggest board (up to `MAX_BOARD`) that fits a terminal of
/// `cols`×`rows`: 2 columns per cell plus border, and border + status line.
fn board_for(cols: u16, rows: u16) -> Board {
    let w = (cols.saturating_sub(2) / 2) as i16;
    let h = rows.saturating_sub(3) as i16;
    Board {
        width: w.clamp(MIN_BOARD.width, MAX_BOARD.width),
        height: h.clamp(MIN_BOARD.height, MAX_BOARD.height),
    }
}

/// Restores the terminal on drop (normal exit and ordinary unwinds).
struct TerminalGuard;

impl TerminalGuard {
    fn enter(out: &mut Stdout) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(out, terminal::EnterAlternateScreen, cursor::Hide, Clear(ClearType::All))?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn restore_terminal() {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
}

/// Restore the terminal *before* the default hook prints the panic message,
/// otherwise it's printed to the alternate screen and wiped.
fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default(info);
    }));
}

fn high_score_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".snake-game").join("highscore"))
}

fn load_high_score() -> u32 {
    high_score_path()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_high_score(score: u32) {
    if let Some(path) = high_score_path() {
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let _ = fs::write(path, format!("{score}\n"));
    }
}

enum Action {
    Continue,
    Quit,
}

fn handle_key(game: &mut Game, key: KeyEvent) -> Action {
    if key.kind != KeyEventKind::Press {
        return Action::Continue;
    }
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => return Action::Quit,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Action::Quit,
        KeyCode::Up => game.queue_direction(Direction::Up),
        KeyCode::Down => game.queue_direction(Direction::Down),
        KeyCode::Left => game.queue_direction(Direction::Left),
        KeyCode::Right => game.queue_direction(Direction::Right),
        KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => game.toggle_pause(),
        KeyCode::Char('r') | KeyCode::Char('R')
            if matches!(game.state, GameState::GameOver | GameState::Won) =>
        {
            game.restart()
        }
        _ => {}
    }
    Action::Continue
}

fn run(out: &mut Stdout) -> io::Result<()> {
    let mut size = terminal::size()?;
    let mut game = Game::new(board_for(size.0, size.1), StdRng::from_os_rng());
    let mut high_score = load_high_score();
    let need = render::required_size(&game);

    let fits = |size: (u16, u16)| size.0 >= need.0 && size.1 >= need.1;
    let mut too_small = !fits(size);
    // Whether *we* paused the game because of a resize (so we know to resume).
    let mut auto_paused = false;
    if too_small && game.state == GameState::Running {
        game.state = GameState::Paused;
        auto_paused = true;
    }

    let mut next_tick = Instant::now() + game.tick_rate;
    let mut dirty = true;

    loop {
        if dirty {
            if too_small {
                render::draw_too_small(out, need, size)?;
            } else {
                render::draw(out, &game, high_score.max(game.score))?;
            }
            dirty = false;
        }

        // Drain input until the tick budget is spent.
        let timeout = next_tick.saturating_duration_since(Instant::now());
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    let before = game.state;
                    if let Action::Quit = handle_key(&mut game, key) {
                        break;
                    }
                    if game.state != before {
                        auto_paused = false;
                        if before != GameState::Paused && game.state == GameState::Running {
                            // Restarted: give the new round a full first tick.
                            next_tick = Instant::now() + game.tick_rate;
                        }
                        dirty = true;
                    }
                }
                Event::Resize(w, h) => {
                    size = (w, h);
                    let was_small = too_small;
                    too_small = !fits(size);
                    if too_small && game.state == GameState::Running {
                        game.state = GameState::Paused;
                        auto_paused = true;
                    } else if !too_small && auto_paused {
                        game.state = GameState::Running;
                        auto_paused = false;
                    }
                    if was_small != too_small {
                        execute!(out, Clear(ClearType::All))?;
                    }
                    dirty = true;
                }
                _ => {}
            }
            continue;
        }

        // Tick boundary.
        let prev_state = game.state;
        game.tick();
        if game.state != prev_state && game.score > high_score {
            high_score = game.score;
            save_high_score(high_score);
        }
        if game.state == GameState::Running || game.state != prev_state {
            dirty = true;
        }
        next_tick = Instant::now() + game.tick_rate.max(Duration::from_millis(1));
    }

    if game.score > high_score {
        save_high_score(game.score);
    }
    Ok(())
}

fn main() -> io::Result<()> {
    install_panic_hook();
    let mut out = io::stdout();
    let _guard = TerminalGuard::enter(&mut out)?;
    let result = run(&mut out);
    out.flush()?;
    result
}
