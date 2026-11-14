mod game;
mod render;
mod snake;
mod theme;

use std::fs;
use std::io::{self, Stdout, Write};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{cursor, execute};
use rand::rngs::StdRng;
use rand::SeedableRng;

use chrono::{Local, NaiveDate};
use game::{Board, Game, GameState};
use render::Look;
use theme::Theme;
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

fn run(out: &mut Stdout, look: Look) -> io::Result<()> {
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
                render::draw(out, &game, high_score.max(game.score), look)?;
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

const USAGE: &str = "\
Usage: snake-game [--theme <name>] [--ascii]

  --theme <name>  Pick a theme. `--theme list` shows them all.
                  Without it, holidays pick their own theme automatically.
                  Use `--theme classic` to turn holiday themes off.
  --ascii         Plain-text food and titles, for terminals that draw emoji badly.
  -h, --help      Show this help.";

#[derive(Debug, PartialEq)]
enum Command {
    Play { theme: Option<&'static Theme>, ascii: bool },
    ListThemes,
    Help,
}

fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Command, String> {
    let mut theme = None;
    let mut ascii = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        let (flag, inline) = match arg.split_once('=') {
            Some((f, v)) => (f.to_string(), Some(v.to_string())),
            None => (arg.clone(), None),
        };
        match flag.as_str() {
            "--theme" | "-t" => {
                let name = inline
                    .or_else(|| args.next())
                    .ok_or("--theme needs a name (try `--theme list`)")?;
                if name == "list" {
                    return Ok(Command::ListThemes);
                }
                theme = Some(theme::by_name(&name).ok_or_else(|| {
                    format!("Unknown theme `{name}`. Try `--theme list`.")
                })?);
            }
            "--ascii" => ascii = true,
            "-h" | "--help" => return Ok(Command::Help),
            _ => return Err(format!("Unknown option: {arg}")),
        }
    }
    Ok(Command::Play { theme, ascii })
}

/// `--theme` wins; otherwise today's holiday, if any; otherwise classic.
fn resolve_theme(chosen: Option<&'static Theme>, today: NaiveDate) -> &'static Theme {
    chosen.or_else(|| theme::holiday_for(today)).unwrap_or(&theme::CLASSIC)
}

/// Left-aligns `s` to `width` terminal columns (emoji count as 2).
fn pad(s: &str, width: usize) -> String {
    use unicode_width::UnicodeWidthStr;
    format!("{s}{}", " ".repeat(width.saturating_sub(s.width())))
}

fn list_themes(today: NaiveDate) {
    let active = resolve_theme(None, today);
    println!("Themes (today: {}):\n", active.name);
    for t in theme::ALL {
        let food: String = t.food.iter().map(|f| f.emoji).collect::<Vec<_>>().join(" ");
        let marker = if *t == active { "▶" } else { " " };
        println!("{marker} {} {} {} {}", pad(t.name, 13), pad(&food, 12), pad(t.when, 38), t.title.trim());
    }
    println!("\nPlay one with: snake-game --theme <name>");
}

fn main() -> io::Result<()> {
    let today = Local::now().date_naive();
    let look = match parse_args(std::env::args().skip(1)) {
        Ok(Command::Play { theme, ascii }) => Look { theme: resolve_theme(theme, today), ascii },
        Ok(Command::ListThemes) => {
            list_themes(today);
            return Ok(());
        }
        Ok(Command::Help) => {
            println!("{USAGE}");
            return Ok(());
        }
        Err(msg) => {
            eprintln!("{msg}\n\n{USAGE}");
            std::process::exit(2);
        }
    };

    install_panic_hook();
    let mut out = io::stdout();
    let _guard = TerminalGuard::enter(&mut out)?;
    let result = run(&mut out, look);
    out.flush()?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Command, String> {
        parse_args(args.iter().map(|s| s.to_string()))
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn no_args_plays_with_auto_theme() {
        assert_eq!(parse(&[]), Ok(Command::Play { theme: None, ascii: false }));
    }

    #[test]
    fn theme_flag_both_spellings() {
        let want = Ok(Command::Play { theme: Some(&theme::HALLOWEEN), ascii: false });
        assert_eq!(parse(&["--theme", "halloween"]), want);
        assert_eq!(parse(&["--theme=halloween"]), want);
        assert_eq!(parse(&["-t", "Halloween"]), want);
    }

    #[test]
    fn ascii_flag() {
        assert_eq!(
            parse(&["--ascii", "--theme", "carnaval"]),
            Ok(Command::Play { theme: Some(&theme::CARNAVAL), ascii: true })
        );
    }

    #[test]
    fn list_help_and_errors() {
        assert_eq!(parse(&["--theme", "list"]), Ok(Command::ListThemes));
        assert_eq!(parse(&["--help"]), Ok(Command::Help));
        assert!(parse(&["--theme"]).is_err());
        assert!(parse(&["--theme", "nope"]).is_err());
        assert!(parse(&["--walls"]).is_err());
    }

    #[test]
    fn flag_beats_holiday_and_classic_turns_holidays_off() {
        let christmas_day = date(2026, 12, 25);
        assert_eq!(resolve_theme(None, christmas_day).name, "christmas");
        assert_eq!(resolve_theme(Some(&theme::CARNAVAL), christmas_day).name, "carnaval");
        assert_eq!(resolve_theme(Some(&theme::CLASSIC), christmas_day).name, "classic");
        assert_eq!(resolve_theme(None, date(2026, 11, 14)).name, "classic");
    }
}
