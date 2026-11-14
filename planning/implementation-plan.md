# Snake Game — Implementation Plan

Terminal-based Snake game written in Rust, controlled with arrow keys.

## Goals

- Playable classic Snake in the terminal.
- Arrow-key controls (Up/Down/Left/Right).
- Smooth fixed-tick game loop, no flicker.
- Score tracking, game-over state, and ability to restart/quit.
- Game logic testable without a terminal.

## Tech Stack

| Concern | Choice | Why |
|---|---|---|
| Language | Rust (stable) | Requested, good fit for a tight game loop |
| Terminal UI / input | [`crossterm`](https://crates.io/crates/crossterm) | Cross-platform raw-mode terminal control, keyboard events, cursor/color |
| Randomness | [`rand`](https://crates.io/crates/rand) | Food placement |
| Build | `cargo` | Standard Rust tooling |

No full TUI framework (e.g. `ratatui`) needed at first — direct `crossterm` drawing is enough for a grid-based game. Can revisit if the UI grows (menus, multiple screens).

**Pin the `rand` major version** in `Cargo.toml` and write against the version pinned. `rand` 0.9 renamed `thread_rng()` → `rng()` and `gen_range()` → `random_range()`; the old names are deprecated aliases, so mixing docs across versions produces warnings.

## Project Structure

```
snake-game/
├── Cargo.toml
├── planning/
│   └── implementation-plan.md
└── src/
    ├── main.rs    # entry point, terminal setup/teardown, panic hook, game loop
    ├── game.rs    # Game state, update logic, collision/food rules, board, food
    ├── snake.rs   # Snake struct: body, direction, movement
    └── render.rs  # Drawing game state to the terminal
```

`Food` and `Board` are one-field structs — they live in `game.rs` rather than their own
modules. Split them out only if they grow real behavior.

## Core Data Model

- `Point { x: i16, y: i16 }` — grid coordinate. Signed so `-1` is representable *before* the
  bounds check runs.
- `Direction { Up, Down, Left, Right }` — with `opposite()` for the reversal guard.
- `Snake`:
  - `body: VecDeque<Point>` (front = head)
  - `direction: Direction` — the last **committed** direction, i.e. the one actually moved
  - `grow()` — don't pop the tail this tick (used after eating food)
  - `advance(new_head: Point)` — push front, pop back unless growing
- `Food { position: Point }`
- `Board { width: i16, height: i16 }` — playable area inside the terminal frame. Signed to
  match `Point` so bounds checks need no casts.
- `GameState { Running, Paused, GameOver, Won }`
- `Game`:
  - `snake: Snake`
  - `food: Food`
  - `board: Board`
  - `score: u32`
  - `state: GameState`
  - `tick_rate: Duration`
  - `input_queue: VecDeque<Direction>` — capacity 2, see "Input handling"
  - `rng: StdRng` — injected/seedable, see "Testing"

## Game Loop

1. Install a panic hook (see "Terminal restoration"), then enter raw mode, hide cursor, enter
   alternate screen (`crossterm::terminal`).
2. Loop at a fixed tick interval (e.g. 150ms, speeds up as score increases):
   a. Drain input with `crossterm::event::poll`, using a timeout equal to the remaining tick
      budget, and keep polling until that budget is exhausted — several events can arrive
      within one tick (key auto-repeat especially).
   b. Arrow keys push onto `input_queue`; `Resize` events are handled per "Resize handling";
      `p` toggles pause; `q`/`Esc` quits.
   c. On tick boundary: pop at most one direction off `input_queue` and commit it, compute the
      new head position, check collisions, update snake/food/score, re-render.
   d. If `GameOver` or `Won`, show the end screen and wait for `r` (restart) or `q`/`Esc` (quit).

### Terminal restoration

Restore terminal state (raw mode off, show cursor, leave alternate screen) on **every** exit
path, via two mechanisms:

- A `struct TerminalGuard` with `Drop` for the normal path and ordinary unwinds.
- A `std::panic::set_hook` that restores the terminal *before* delegating to the default hook.

The guard alone is not enough: `Drop` does not run under `panic = "abort"`, and a panic message
printed while the alternate screen is still active is wiped the moment we leave it — producing a
silent exit with no diagnostic.

## Collision & Rules

- **Wall collision**: head goes outside board bounds → game over (classic mode; a "wrap-around"
  variant can be a later option).
- **Self collision**: pop the tail *first*, then test the new head against the remaining body.
  When the snake isn't growing, the tail vacates its cell on the same tick, so moving into the
  cell the tail currently occupies is legal — and is what classic Snake does. Checking against
  the full pre-move body kills the player on legitimate tight turns.
- **Food collision**: new head position equals food position → grow snake by 1, increment score,
  respawn food (see below).
- **Win condition**: if the snake occupies every cell there is nowhere to respawn food →
  `GameState::Won`. Without this the game hangs on a perfect run.

### Food respawn

Enumerate the empty cells and pick one by index, rather than rejection-sampling random
coordinates until one misses the snake. Rejection sampling degrades as the board fills and
never terminates once the board is full. Enumeration also makes the win condition fall out
naturally: an empty candidate list *is* the win.

### Input handling

- `input_queue` holds up to **2** buffered directions. A single pending slot drops the second
  of a fast two-tap turn (e.g. up-then-right), which reads to players as unresponsive input;
  a 2-deep queue applies them on consecutive ticks.
- On enqueue, a direction is rejected if it reverses (or repeats) the **last queued** direction
  (or the committed one when the queue is empty). Because each queued direction commits on its
  own tick, Right → Up → Left is a safe U-turn, not a reversal.
- At commit time, the popped direction is re-checked against the **last committed** direction
  and discarded if it would reverse. This is the authoritative guard.
- *(Revised during implementation: the original rule — reject anything opposite to the committed
  direction at enqueue — silently dropped legitimate fast U-turns such as Up → Right → Down.)*

## Rendering

- Clear & redraw each tick (double-buffer the frame as a string, then write it in one
  `queue!`/`flush` to avoid flicker).
- **Each logical cell renders two characters wide.** Terminal cells are roughly 1:2
  (width:height), so a cell-square board looks like a tall rectangle and the snake appears to
  move faster horizontally than vertically. The 2:1 mapping lives in the `Point` → screen
  coordinate conversion in `render.rs` — cheaper to decide now than to retrofit.
- Draw a bordered box for the board.
- Snake head vs. body with distinct characters/colors (e.g. `@` head, `o` body, `*` food).
- Status line: current score, and controls hint (`Arrows to move, P to pause, Q to quit`).
- End screen: final score, "Press R to restart, Q to quit".

### Resize handling

Consume `crossterm::event::Event::Resize`. If the terminal is smaller than the board plus
border and status line, transition to `Paused` and draw a "terminal too small (need N×M)"
screen; resume when it grows back. Otherwise drawing runs past the viewport and corrupts the
frame.

## Testing

The valuable logic here is pure, so keep it that way:

- `Game::update()` performs no IO — it takes an optional committed direction and advances state.
  `render.rs` is the only module that touches the terminal.
- **Inject the RNG** (a seeded `StdRng`, or generic over `impl Rng`) instead of calling
  `rand::rng()` inside food placement. Food becomes deterministic and a whole game is
  reproducible from a seed.

Tests worth writing from milestone 3 onward:

- Wall collision at each of the four bounds.
- Moving into the current tail cell is legal when not growing (the rule above).
- The two-tap reversal is rejected (`Right` → `Up` → `Left` within one tick).
- A buffered two-tap turn is applied across two consecutive ticks, not dropped.
- Eating food increases length by exactly 1 and score by exactly 1.
- Food never spawns on a body segment (seeded, over many respawns).
- A board filled by the snake yields `Won`, not a hang.

## Milestones

1. **Project scaffold** — `cargo new`, add `crossterm` + `rand` deps, panic hook + terminal
   guard, empty game loop that clears the screen and exits on `q`.
2. **Static rendering** — board border, snake at a fixed position, food at a fixed position,
   2:1 cell mapping in place.
3. **Movement** — snake moves each tick in its current direction; wall collision ends the game.
   First unit tests land here.
4. **Input handling** — arrow keys change direction (reversal guard against committed direction,
   2-deep buffer).
5. **Food & growth** — eating food grows the snake, respawns food by empty-cell enumeration,
   increments score; win state when the board fills.
6. **Self-collision & end screens** — tail-aware collision check, game-over/win UI,
   restart/quit.
7. **Pause & resize** — `p` toggles pause; resize events pause with a "too small" screen.
8. **Polish** — difficulty scaling (speed increases with score), colors, high-score persistence
   to `~/.snake-game/highscore`.

## Open Questions / Options for Later

- Wrap-around walls vs. hard walls (toggle?).
- Configurable board size via CLI args (`clap`) if desired.
- Whether the high-score file should move to a platform data dir (`directories` crate) instead
  of `~/.snake-game/`.

## Out of Scope (for v1)

- Multiplayer.
- Sound.
- Mouse support.
- Networked leaderboard.
