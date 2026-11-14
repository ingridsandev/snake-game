# 🐍 snake-game

> *Eat. Grow. Don't bite yourself. Repeat until your terminal is 90% snake.*

A tiny, fast, flicker-free Snake game that lives in your terminal. Written in Rust,
because even a snake deserves memory safety.

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│          ████████████████                                │
│                        ██                ◆               │
│                        ████████████                      │
│                                                          │
└──────────────────────────────────────────────────────────┘
 Score: 7  Best: 42   Arrows move · P pause · Q quit
```

## 🚀 Play in 10 seconds

```bash
git clone https://github.com/ingridsandev/snake-game.git
cd snake-game
cargo run --release
```

No config. No accounts. No loot boxes. Just you, a snake, and your poor decisions.

## 🎮 Controls

| Key | What it does |
|---|---|
| `↑` `↓` `←` `→` | Slither |
| `P` / `Space` | Pause (snakes need snack breaks too) |
| `R` | Rise from the dead after game over |
| `Q` / `Esc` | Rage quit, gracefully |

## 🧠 Things this snake is surprisingly smart about

- **Square cells, honest speed.** Each cell is two characters wide, so the snake moves just as
  fast sideways as it does up and down. No more mysterious "why is it so fast sideways?!"
- **Fast fingers welcome.** Hit two arrows in one tick and both turns happen, one per tick.
  Quick U-turns work; you will never accidentally do a 180 into your own neck.
- **Chasing your tail is legal.** The tail moves out of the way on the same tick, so you can
  follow it as tightly as you like, just like the real thing.
- **You can actually win.** Fill the entire board and you get a victory screen instead of an
  infinite loop. (Good luck.)
- **Edges are portals.** Leave the board on one side and come back on the other. Your own body
  is still on the other side, though, so look before you leap.
- **It gets faster.** Every snack shaves a few milliseconds off the tick. Starts at a chill
  150ms, tops out at a frantic 60ms.
- **Fits your terminal.** The board sizes itself to your window (up to 30×20). Shrink the
  window mid-game and it politely pauses until you make room again.
- **Never wrecks your terminal.** Quit, crash, or panic: your terminal comes back exactly as
  you left it.
- **Remembers your glory.** High score is saved to `~/.snake-game/highscore`, so you can
  brag later.

## 🧪 Tests

The game rules are pure Rust with zero terminal IO, so they're easy to test:

```bash
cargo test
```

Covers portal edges, self-bites, tail-chasing, sneaky double-tap reversals, food that never spawns
inside you, and the elusive full-board win.

## 🗂️ What's where

```
src/
├── main.rs    # terminal setup, game loop, high score, "please don't break my terminal"
├── game.rs    # the rules: movement, collisions, food, winning, speed
├── snake.rs   # the noodle itself
└── render.rs  # turns state into pretty boxes
planning/
└── implementation-plan.md   # how it was designed, milestone by milestone
```

## 🛣️ Maybe someday

- Board size from the command line
- Something even fancier than a high score file

---

Made with 🦀 and an unhealthy amount of arrow-key mashing.
