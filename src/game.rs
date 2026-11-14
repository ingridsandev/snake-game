use std::collections::VecDeque;
use std::time::Duration;

use rand::rngs::StdRng;
use rand::Rng;

use crate::snake::{Direction, Point, Snake};

pub const INITIAL_LEN: usize = 3;
const INPUT_QUEUE_CAP: usize = 2;
const BASE_TICK_MS: u64 = 150;
const MIN_TICK_MS: u64 = 60;
const TICK_STEP_MS: u64 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub width: i16,
    pub height: i16,
}

impl Board {
    #[cfg(test)]
    pub fn contains(&self, p: Point) -> bool {
        p.x >= 0 && p.y >= 0 && p.x < self.width && p.y < self.height
    }

    /// Wraps a point that stepped off one edge onto the opposite edge.
    pub fn wrap(&self, p: Point) -> Point {
        Point::new(p.x.rem_euclid(self.width), p.y.rem_euclid(self.height))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Food {
    pub position: Point,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Running,
    Paused,
    GameOver,
    Won,
}

pub struct Game {
    pub snake: Snake,
    pub food: Food,
    pub board: Board,
    pub score: u32,
    pub state: GameState,
    pub tick_rate: Duration,
    pub input_queue: VecDeque<Direction>,
    pub rng: StdRng,
}

impl Game {
    pub fn new(board: Board, rng: StdRng) -> Self {
        let head = Point::new(board.width / 2, board.height / 2);
        let snake = Snake::new(head, INITIAL_LEN, Direction::Right);
        let mut game = Self {
            snake,
            food: Food { position: head },
            board,
            score: 0,
            state: GameState::Running,
            tick_rate: Duration::from_millis(BASE_TICK_MS),
            input_queue: VecDeque::with_capacity(INPUT_QUEUE_CAP),
            rng,
        };
        if !game.respawn_food() {
            game.state = GameState::Won;
        }
        game
    }

    /// Starts a fresh round on the same board, keeping the RNG stream.
    pub fn restart(&mut self) {
        let rng = self.rng.clone();
        *self = Game::new(self.board, rng);
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Running => GameState::Paused,
            GameState::Paused => GameState::Running,
            other => other,
        };
    }

    /// Buffers a direction change. Each queued direction commits on its own
    /// tick, so on enqueue it only has to be valid relative to the one before
    /// it (that keeps fast U-turns like Up → Right → Down working). The
    /// authoritative reversal guard against the *committed* direction runs at
    /// commit time in `next_direction`.
    pub fn queue_direction(&mut self, dir: Direction) {
        if self.state != GameState::Running || self.input_queue.len() >= INPUT_QUEUE_CAP {
            return;
        }
        let last = self.input_queue.back().copied().unwrap_or(self.snake.direction());
        if dir == last.opposite() || dir == last {
            return;
        }
        self.input_queue.push_back(dir);
    }

    /// Pops the next valid buffered direction. A buffered entry can become a
    /// reversal once an earlier one commits (Right → Up, Down), so re-check.
    fn next_direction(&mut self) -> Option<Direction> {
        let committed = self.snake.direction();
        while let Some(dir) = self.input_queue.pop_front() {
            if dir != committed.opposite() && dir != committed {
                return Some(dir);
            }
        }
        None
    }

    /// One game tick: commit at most one buffered direction and advance.
    pub fn tick(&mut self) {
        if self.state != GameState::Running {
            return;
        }
        let dir = self.next_direction();
        self.update(dir);
    }

    /// Advances the game by one step. Pure: no IO.
    pub fn update(&mut self, dir: Option<Direction>) {
        if self.state != GameState::Running {
            return;
        }
        let dir = match dir {
            Some(d) if d != self.snake.direction().opposite() => d,
            _ => self.snake.direction(),
        };
        // Edges wrap: stepping off one side re-enters on the opposite side.
        let new_head = self.board.wrap(self.snake.head().step(dir));

        let eating = new_head == self.food.position;
        if self.snake.would_collide(new_head, eating) {
            self.state = GameState::GameOver;
            return;
        }

        if eating {
            self.snake.grow();
        }
        self.snake.advance(new_head, dir);

        if eating {
            self.score += 1;
            self.tick_rate = tick_rate_for(self.score);
            if !self.respawn_food() {
                self.state = GameState::Won;
            }
        }
    }

    /// Places food on a uniformly random empty cell. Returns `false` when
    /// there is no empty cell left (the snake fills the board).
    fn respawn_food(&mut self) -> bool {
        let empty: Vec<Point> = (0..self.board.height)
            .flat_map(|y| (0..self.board.width).map(move |x| Point::new(x, y)))
            .filter(|&p| !self.snake.contains(p))
            .collect();
        if empty.is_empty() {
            return false;
        }
        self.food.position = empty[self.rng.random_range(0..empty.len())];
        true
    }
}

fn tick_rate_for(score: u32) -> Duration {
    let ms = BASE_TICK_MS.saturating_sub(score as u64 * TICK_STEP_MS).max(MIN_TICK_MS);
    Duration::from_millis(ms)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn game(w: i16, h: i16) -> Game {
        Game::new(Board { width: w, height: h }, StdRng::seed_from_u64(42))
    }

    fn with_snake(w: i16, h: i16, points: &[Point], dir: Direction) -> Game {
        let mut g = game(w, h);
        g.snake = Snake::from_points(points, dir);
        // Park food somewhere out of the way of the test's moves.
        g.food.position = Point::new(w - 1, h - 1);
        g
    }

    fn p(x: i16, y: i16) -> Point {
        Point::new(x, y)
    }

    #[test]
    fn wraps_right_to_left() {
        let mut g = with_snake(5, 5, &[p(4, 2), p(3, 2)], Direction::Right);
        g.update(None);
        assert_eq!(g.state, GameState::Running);
        assert_eq!(g.snake.head(), p(0, 2));
    }

    #[test]
    fn wraps_left_to_right() {
        let mut g = with_snake(5, 5, &[p(0, 2), p(1, 2)], Direction::Left);
        g.update(None);
        assert_eq!(g.state, GameState::Running);
        assert_eq!(g.snake.head(), p(4, 2));
    }

    #[test]
    fn wraps_top_to_bottom() {
        let mut g = with_snake(5, 5, &[p(2, 0), p(2, 1)], Direction::Up);
        g.update(None);
        assert_eq!(g.state, GameState::Running);
        assert_eq!(g.snake.head(), p(2, 4));
    }

    #[test]
    fn wraps_bottom_to_top() {
        let mut g = with_snake(5, 5, &[p(2, 4), p(2, 3)], Direction::Down);
        g.update(None);
        assert_eq!(g.state, GameState::Running);
        assert_eq!(g.snake.head(), p(2, 0));
    }

    #[test]
    fn wrapping_into_own_body_still_kills() {
        // Head at the right edge; the body sits on the left edge of that row.
        let mut g = with_snake(4, 3, &[p(3, 1), p(3, 0), p(2, 0), p(1, 0), p(0, 0), p(0, 1), p(0, 2)], Direction::Down);
        g.food.position = p(1, 2);
        g.update(Some(Direction::Right)); // wraps to (0,1): a body segment
        assert_eq!(g.state, GameState::GameOver);
    }

    #[test]
    fn wrapping_onto_food_eats_it() {
        let mut g = with_snake(5, 5, &[p(4, 2), p(3, 2)], Direction::Right);
        g.food.position = p(0, 2);
        g.update(None);
        assert_eq!(g.score, 1);
        assert_eq!(g.snake.len(), 3);
    }

    #[test]
    fn moving_into_tail_cell_is_legal_when_not_growing() {
        // 2x2 loop: head (1,0) moving Down → (1,1), then Left → (0,1) which is the tail.
        let mut g = with_snake(
            6,
            6,
            &[p(1, 0), p(0, 0), p(0, 1), p(1, 1)],
            Direction::Right,
        );
        g.food.position = p(5, 5);
        // Head (1,0) moving Down into (1,1), currently the tail.
        g.update(Some(Direction::Down));
        assert_eq!(g.state, GameState::Running);
        assert_eq!(g.snake.head(), p(1, 1));
        assert_eq!(g.snake.len(), 4);
    }

    #[test]
    fn moving_into_tail_cell_kills_when_growing() {
        let mut g = with_snake(6, 6, &[p(1, 0), p(0, 0), p(0, 1), p(1, 1)], Direction::Right);
        g.food.position = p(1, 1); // food on the tail cell: tail won't vacate
        g.update(Some(Direction::Down));
        assert_eq!(g.state, GameState::GameOver);
    }

    #[test]
    fn self_collision_into_body() {
        // Head at (2,1) moving Left into (1,1), a mid-body segment.
        let mut g = with_snake(
            6,
            6,
            &[p(2, 1), p(2, 0), p(1, 0), p(1, 1), p(1, 2)],
            Direction::Down,
        );
        g.update(Some(Direction::Left));
        assert_eq!(g.state, GameState::GameOver);
    }

    #[test]
    fn direct_reversal_is_rejected() {
        let mut g = with_snake(10, 10, &[p(5, 5), p(4, 5), p(3, 5)], Direction::Right);
        g.queue_direction(Direction::Left);
        assert!(g.input_queue.is_empty());
        g.tick();
        assert_eq!(g.snake.head(), p(6, 5));
        assert_eq!(g.state, GameState::Running);
    }

    #[test]
    fn two_tap_reversal_never_reverses_into_neck() {
        // Moving Right, tap Up then Left within one tick: they commit on
        // separate ticks, so the snake turns Up, then Left — never straight back.
        let mut g = with_snake(10, 10, &[p(5, 5), p(4, 5), p(3, 5)], Direction::Right);
        g.queue_direction(Direction::Up);
        g.queue_direction(Direction::Left);
        g.tick();
        assert_eq!(g.snake.head(), p(5, 4));
        g.tick();
        assert_eq!(g.snake.head(), p(4, 4));
        assert_eq!(g.state, GameState::Running);
    }

    #[test]
    fn buffered_reversal_is_rejected_at_commit() {
        // Right, tap Up then Down: Down isn't opposite of Right when queued,
        // but becomes a reversal once Up commits.
        let mut g = with_snake(10, 10, &[p(5, 5), p(4, 5), p(3, 5)], Direction::Right);
        g.queue_direction(Direction::Up);
        g.queue_direction(Direction::Down);
        g.tick();
        g.tick();
        assert_eq!(g.state, GameState::Running);
        assert_eq!(g.snake.head(), p(5, 3));
    }

    #[test]
    fn buffered_two_tap_turn_applies_on_consecutive_ticks() {
        // Moving Up, tap Right then Down within one tick (a U-turn).
        let mut g = with_snake(10, 10, &[p(5, 5), p(5, 6), p(5, 7)], Direction::Up);
        g.queue_direction(Direction::Right);
        g.queue_direction(Direction::Down);
        g.tick();
        assert_eq!(g.snake.head(), p(6, 5));
        assert_eq!(g.snake.direction(), Direction::Right);
        g.tick();
        assert_eq!(g.snake.head(), p(6, 6));
        assert_eq!(g.snake.direction(), Direction::Down);
        assert_eq!(g.state, GameState::Running);
    }

    #[test]
    fn eating_food_grows_by_one_and_scores_one() {
        let mut g = with_snake(10, 10, &[p(5, 5), p(4, 5), p(3, 5)], Direction::Right);
        g.food.position = p(6, 5);
        g.update(None);
        assert_eq!(g.snake.len(), 4);
        assert_eq!(g.score, 1);
        assert_ne!(g.food.position, p(6, 5));
        // Next move doesn't grow further.
        g.update(None);
        assert_eq!(g.snake.len(), 4);
        assert_eq!(g.score, 1);
    }

    #[test]
    fn food_never_spawns_on_snake() {
        let mut g = game(8, 8);
        for _ in 0..2000 {
            assert!(g.respawn_food());
            assert!(!g.snake.contains(g.food.position));
            assert!(g.board.contains(g.food.position));
        }
    }

    #[test]
    fn filling_the_board_wins() {
        // 2x2 board, snake occupies 3 cells, food in the last one.
        let mut g = with_snake(2, 2, &[p(0, 1), p(0, 0), p(1, 0)], Direction::Down);
        g.food.position = p(1, 1);
        g.update(Some(Direction::Right));
        assert_eq!(g.snake.len(), 4);
        assert_eq!(g.state, GameState::Won);
    }

    #[test]
    fn speed_increases_with_score() {
        assert_eq!(tick_rate_for(0), Duration::from_millis(150));
        assert!(tick_rate_for(5) < tick_rate_for(0));
        assert_eq!(tick_rate_for(1000), Duration::from_millis(MIN_TICK_MS));
    }

    #[test]
    fn same_seed_same_game() {
        let a = game(20, 20);
        let b = game(20, 20);
        assert_eq!(a.food, b.food);
    }
}
