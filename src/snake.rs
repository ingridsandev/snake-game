use std::collections::VecDeque;

/// A grid coordinate. Signed so out-of-bounds positions (e.g. `-1`) are
/// representable before the bounds check runs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Point {
    pub x: i16,
    pub y: i16,
}

impl Point {
    pub const fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }

    pub fn step(self, direction: Direction) -> Self {
        let (dx, dy) = direction.delta();
        Self::new(self.x + dx, self.y + dy)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    fn delta(self) -> (i16, i16) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Snake {
    /// Front = head.
    body: VecDeque<Point>,
    /// The last *committed* direction, i.e. the one the snake actually moved in.
    direction: Direction,
    growing: bool,
}

impl Snake {
    /// Creates a snake of `len` segments with its head at `head`, trailing
    /// behind it opposite to `direction`.
    pub fn new(head: Point, len: usize, direction: Direction) -> Self {
        let back = direction.opposite();
        let mut body = VecDeque::with_capacity(len);
        let mut p = head;
        for _ in 0..len.max(1) {
            body.push_back(p);
            p = p.step(back);
        }
        Self { body, direction, growing: false }
    }

    /// Builds a snake from explicit segments (head first).
    #[cfg(test)]
    pub fn from_points(points: &[Point], direction: Direction) -> Self {
        Self { body: points.iter().copied().collect(), direction, growing: false }
    }

    pub fn head(&self) -> Point {
        self.body[0]
    }

    pub fn body(&self) -> &VecDeque<Point> {
        &self.body
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.body.len()
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn contains(&self, p: Point) -> bool {
        self.body.contains(&p)
    }

    /// Would moving the head to `p` hit the body? The tail vacates its cell
    /// on the same tick unless the snake is growing, so it doesn't count.
    pub fn would_collide(&self, p: Point, growing: bool) -> bool {
        let checked = if growing { self.body.len() } else { self.body.len() - 1 };
        self.body.iter().take(checked).any(|&s| s == p)
    }

    /// Don't pop the tail on the next `advance`.
    pub fn grow(&mut self) {
        self.growing = true;
    }

    pub fn advance(&mut self, new_head: Point, direction: Direction) {
        self.body.push_front(new_head);
        if self.growing {
            self.growing = false;
        } else {
            self.body.pop_back();
        }
        self.direction = direction;
    }
}
