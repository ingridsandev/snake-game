use std::io::{self, Write};

use crossterm::cursor::MoveTo;
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor, Stylize};
use crossterm::terminal::{Clear, ClearType};
use crossterm::queue;

use crate::game::{Game, GameState};
use crate::snake::Point;

/// Each logical cell is drawn this many columns wide, so the board looks
/// square in a terminal whose cells are roughly 1:2 (width:height).
const CELL_W: u16 = 2;

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

pub fn draw<W: Write>(out: &mut W, game: &Game, high_score: u32) -> io::Result<()> {
    let w = game.board.width as usize * CELL_W as usize;
    let h = game.board.height as u16;

    // Border + empty interior, row by row (overwrites last frame, no flicker).
    let horiz = "─".repeat(w);
    queue!(out, MoveTo(0, 0), SetForegroundColor(Color::DarkGrey), Print(format!("┌{horiz}┐")))?;
    let blank = " ".repeat(w);
    for row in 1..=h {
        queue!(out, MoveTo(0, row), Print(format!("│{blank}│")))?;
    }
    queue!(out, MoveTo(0, h + 1), Print(format!("└{horiz}┘")), ResetColor)?;

    // Food.
    let (fx, fy) = to_screen(game.food.position);
    if game.state != GameState::Won {
        queue!(out, MoveTo(fx, fy), Print("◆ ".red().bold()))?;
    }

    // Snake: body first, head last so it's always on top.
    for seg in game.snake.body().iter().skip(1) {
        let (x, y) = to_screen(*seg);
        queue!(out, MoveTo(x, y), Print("██".green()))?;
    }
    let (hx, hy) = to_screen(game.snake.head());
    let head_color = if game.state == GameState::GameOver { Color::Red } else { Color::Yellow };
    queue!(out, MoveTo(hx, hy), Print("██".with(head_color)))?;

    // Overlays.
    match game.state {
        GameState::Running => {}
        GameState::Paused => overlay(out, game, &["PAUSED", "", "P to resume"])?,
        GameState::GameOver => overlay(
            out,
            game,
            &["GAME OVER", "", &format!("Score: {}", game.score), "", "R restart   Q quit"],
        )?,
        GameState::Won => overlay(
            out,
            game,
            &["YOU WIN!", "", &format!("Score: {}", game.score), "", "R restart   Q quit"],
        )?,
    }

    // Status line.
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
    let box_w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) + 4;
    let box_w = box_w.min(inner_w);
    let left = 1 + (inner_w - box_w) / 2;
    let top = 1 + (game.board.height as usize).saturating_sub(lines.len() + 2) / 2;

    let edge = " ".repeat(box_w);
    queue!(out, MoveTo(left as u16, top as u16), Print(edge.as_str().on_dark_grey()))?;
    for (i, line) in lines.iter().enumerate() {
        let pad = box_w - line.chars().count();
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
