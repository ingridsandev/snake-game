use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::queue;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor, Stylize};
use crossterm::terminal::{Clear, ClearType};
use unicode_width::UnicodeWidthStr;

use crate::game::{Game, GameState};
use crate::snake::Point;
use crate::theme::{FoodGlyph, Theme};

/// Each logical cell is drawn this many columns wide, so the board looks
/// square in a terminal whose cells are roughly 1:2 (width:height).
const CELL_W: u16 = 2;

/// How the game looks: which theme, and whether to avoid emoji.
#[derive(Clone, Copy)]
pub struct Look {
    pub theme: &'static Theme,
    pub ascii: bool,
}

/// Terminal size needed to draw `game`: board + border + status line.
pub fn required_size(game: &Game) -> (u16, u16) {
    let cols = game.board.width as u16 * CELL_W + 2;
    let rows = game.board.height as u16 + 2 + 1;
    (cols, rows)
}

/// Maps a logical grid point to a terminal (column, row), inside the border.
fn to_screen(p: Point) -> (u16, u16) {
    (1 + p.x as u16 * CELL_W, 1 + p.y as u16)
}

/// Picks which of the theme's foods to show. Keyed on the food's position so
/// it stays the same while that food sits there, and varies between spawns.
fn food_glyph(theme: &'static Theme, p: Point) -> &'static FoodGlyph {
    let i = (p.x as usize).wrapping_mul(31).wrapping_add(p.y as usize * 17);
    &theme.food[i % theme.food.len()]
}

/// Top border with the theme title set into it: `┌─ 🎃 Spooky Snake ───┐`.
fn top_border(title: &str, inner_w: usize) -> String {
    let title_w = title.width();
    if title_w + 2 > inner_w {
        return format!("┌{}┐", "─".repeat(inner_w));
    }
    format!("┌─{title}{}┐", "─".repeat(inner_w - 1 - title_w))
}

pub fn draw<W: Write>(out: &mut W, game: &Game, high_score: u32, look: Look) -> io::Result<()> {
    let theme = look.theme;
    let w = game.board.width as usize * CELL_W as usize;
    let h = game.board.height as u16;

    // Border + empty interior, row by row (overwrites last frame, no flicker).
    let title = if look.ascii { theme.title_ascii } else { theme.title };
    queue!(out, MoveTo(0, 0), SetForegroundColor(theme.border), Print(top_border(title, w)))?;
    let blank = " ".repeat(w);
    for row in 1..=h {
        queue!(out, MoveTo(0, row), Print(format!("│{blank}│")))?;
    }
    queue!(out, MoveTo(0, h + 1), Print(format!("└{}┘", "─".repeat(w))), ResetColor)?;

    // Food.
    if game.state != GameState::Won {
        let (fx, fy) = to_screen(game.food.position);
        let glyph = food_glyph(theme, game.food.position);
        if look.ascii {
            queue!(out, MoveTo(fx, fy), Print(glyph.ascii.with(glyph.color).bold()))?;
        } else {
            queue!(out, MoveTo(fx, fy), Print(glyph.emoji))?;
        }
    }

    // Snake: body first (striped), head last so it's always on top.
    for (i, seg) in game.snake.body().iter().enumerate().skip(1) {
        let (x, y) = to_screen(*seg);
        let color = theme.body[(i - 1) % theme.body.len()];
        queue!(out, MoveTo(x, y), Print("██".with(color)))?;
    }
    let (hx, hy) = to_screen(game.snake.head());
    let head_color = if game.state == GameState::GameOver { Color::Red } else { theme.head };
    queue!(out, MoveTo(hx, hy), Print("██".with(head_color)))?;

    // Overlays.
    let score = format!("Score: {}", game.score);
    match game.state {
        GameState::Running => {}
        GameState::Paused => overlay(out, game, &["PAUSED", "", "P to resume"])?,
        GameState::GameOver => overlay(
            out,
            game,
            &["GAME OVER", theme.game_over, "", &score, "", "R restart   Q quit"],
        )?,
        GameState::Won => overlay(
            out,
            game,
            &["YOU WIN!", theme.win, "", &score, "", "R restart   Q quit"],
        )?,
    }

    // Status line, truncated to the board's width.
    let status = format!(
        " Score: {}  Best: {}   Arrows move · P pause · Q quit",
        game.score, high_score
    );
    let status: String = status.chars().take(w + 2).collect();
    queue!(
        out,
        MoveTo(0, h + 2),
        Clear(ClearType::CurrentLine),
        Print(status.bold())
    )?;

    out.flush()
}

/// Draws centered lines in the middle of the board.
fn overlay<W: Write>(out: &mut W, game: &Game, lines: &[&str]) -> io::Result<()> {
    let inner_w = game.board.width as usize * CELL_W as usize;
    let box_w = lines.iter().map(|l| l.width()).max().unwrap_or(0) + 4;
    let box_w = box_w.min(inner_w);
    let left = 1 + (inner_w - box_w) / 2;
    let top = 1 + (game.board.height as usize).saturating_sub(lines.len() + 2) / 2;

    let edge = " ".repeat(box_w);
    queue!(out, MoveTo(left as u16, top as u16), Print(edge.as_str().on_dark_grey()))?;
    for (i, line) in lines.iter().enumerate() {
        let pad = box_w.saturating_sub(line.width());
        let text = format!("{}{}{}", " ".repeat(pad / 2), line, " ".repeat(pad - pad / 2));
        queue!(
            out,
            MoveTo(left as u16, (top + 1 + i) as u16),
            Print(text.white().bold().on_dark_grey())
        )?;
    }
    queue!(
        out,
        MoveTo(left as u16, (top + 1 + lines.len()) as u16),
        Print(edge.as_str().on_dark_grey())
    )?;
    Ok(())
}

pub fn draw_too_small<W: Write>(out: &mut W, need: (u16, u16), have: (u16, u16)) -> io::Result<()> {
    queue!(
        out,
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print("Terminal too small".red().bold()),
        MoveTo(0, 1),
        Print(format!("need {}×{}, have {}×{}", need.0, need.1, have.0, have.1)),
        MoveTo(0, 2),
        Print("Resize to continue, Q to quit"),
    )?;
    out.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::ALL;

    #[test]
    fn top_border_keeps_board_width_for_every_title() {
        for theme in ALL {
            for title in [theme.title, theme.title_ascii] {
                for inner_w in [20, 40, 60] {
                    let b = top_border(title, inner_w);
                    assert_eq!(b.width(), inner_w + 2, "{title:?} at {inner_w}");
                }
            }
        }
    }
}
