use components::position::Position;

pub mod combat;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn offset(&self) -> (i32, i32) {
        use self::Direction::*;
        match *self {
            Up => (0, -1),
            Down => (0, 1),
            Left => (-1, 0),
            Right => (1, 0),
        }
    }
}

pub fn distance2(a: (usize, usize), b: (usize, usize)) -> usize {
    ((a.0 as i32 - b.0 as i32).pow(2) + (a.1 as i32 - b.1 as i32).pow(2)) as usize
}

// Counterclockwise from +x axis
enum Octant {
    O0,
    O1,
    O2,
    O3,
    O4,
    O5,
    O6,
    O7,
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
struct SignedPos {
    x: i32,
    y: i32,
}

impl Octant {
    fn transform(&self, SignedPos{ x, y }: SignedPos) -> SignedPos {
        use self::Octant::*;
        match *self {
            O0 => SignedPos { x: x, y: y },
            O1 => SignedPos { x: y, y: x },
            O2 => SignedPos { x: y, y: -x },
            O3 => SignedPos { x: -x, y: y },
            O4 => SignedPos { x: -x, y: -y },
            O5 => SignedPos { x: -y, y: -x },
            O6 => SignedPos { x: -y, y: x },
            O7 => SignedPos { x: x, y: -y },
        }
    }

    fn invert(&self, SignedPos { x, y }: SignedPos) -> SignedPos {
        use self::Octant::*;
        match *self {
            O0 => SignedPos { x: x, y: y },
            O1 => SignedPos { x: y, y: x },
            O2 => SignedPos { x: -y, y: x },
            O3 => SignedPos { x: -x, y: y },
            O4 => SignedPos { x: -x, y: -y },
            O5 => SignedPos { x: -y, y: -x },
            O6 => SignedPos { x: y, y: -x },
            O7 => SignedPos { x: x, y: -y },
        }
    }
}

pub fn bresenham(start: Position, end: Position) -> Vec<Position> {
    use std::cmp::{min, max};

    let mut result = Vec::new();
    // Vertical line special case
    if start.x == end.x {
        for y in min(start.y, end.y)..max(start.y, end.y) + 1 {
            result.push(Position { x: start.x, y: y });
        }
    }

    // Figure out the octant
    let horiz_proj = i32::abs(end.x as i32 - start.x as i32);
    let vert_proj = i32::abs(end.y as i32 - start.y as i32);

    let octant = if end.x > start.x {
        // 0/1/6/7
        if end.y > start.y {
            // 0/1
            if horiz_proj > vert_proj {
                Octant::O0
            }
            else {
                Octant::O1
            }
        }
        else {
            // 6/7
            if horiz_proj > vert_proj {
                Octant::O7
            }
            else {
                Octant::O6
            }
        }
    }
    else {
        // 2/3/4/5
        if end.y > start.y {
            // 2/3
            if horiz_proj > vert_proj {
                Octant::O3
            }
            else {
                Octant::O2
            }
        }
        else {
            // 4/5
            if horiz_proj > vert_proj {
                Octant::O4
            }
            else {
                Octant::O5
            }
        }
    };

    let end = octant.transform(SignedPos { x: end.x as i32 - start.x as i32,
                                           y: end.y as i32 - start.y as i32 });

    let dx = end.x as f32;
    let dy = end.y as f32;
    let derr = f32::abs(dy / dx);
    let mut err = derr - 0.5;
    let mut y = 0;

    for x in 0..end.x + 1 {
        let offset = octant.invert(SignedPos { x: x, y: y });
        let pos = Position {
            x: (start.x as i32 + offset.x) as usize,
            y: (start.y as i32 + offset.y) as usize,
        };
        result.push(pos);
        err += derr;

        if err >= 0.5 {
            y += 1;
            err -= 1.0;
        }
    }

    result
}
