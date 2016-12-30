use specs;
use voodoo;
use voodoo::color::ColorValue;
use voodoo::overlay::Overlay;
use voodoo::window::TermCell;

use super::map::Map;
use super::position::Position;

pub struct StaticDrawable {
    pub tc: TermCell,
}

pub struct LineDrawable {
    pub start: Position,
    pub end: Position,
}

/// Mark an entity as rendering drawables. Use in conjunction with
/// MapRender.
pub struct DrawableRender {
    overlay: Overlay,
}

pub struct RenderSystem {

}

impl StaticDrawable {

}

impl specs::Component for StaticDrawable {
    type Storage = specs::VecStorage<StaticDrawable>;
}

impl specs::Component for LineDrawable {
    type Storage = specs::VecStorage<LineDrawable>;
}

impl super::input::OffsetMovable for LineDrawable {
    fn move_by(&mut self, offset: (i32, i32), map: &mut Map) -> Result<(), ()> {
        use std::cmp::min;

        let new_x = self.end.x as i32 + offset.0;
        let new_y = self.end.y as i32 + offset.1;

        let new_x = if new_x < 0 { 0 } else { min(new_x as usize, map.width) };
        let new_y = if new_y < 0 { 0 } else { min(new_y as usize, map.height) };

        self.end.x = new_x;
        self.end.y = new_y;

        Ok(())
    }
}

impl DrawableRender {
    pub fn new(overlay: Overlay) -> DrawableRender {
        DrawableRender {
            overlay: overlay,
        }
    }

    // TODO: move this fn into a Trait?
    pub fn refresh(&self, compositor: &mut voodoo::compositor::Compositor) {
        self.overlay.refresh(compositor);
    }
}

impl specs::Component for DrawableRender {
    type Storage = specs::VecStorage<DrawableRender>;
}

impl RenderSystem {
    pub fn new() -> RenderSystem {
        RenderSystem {}
    }
}

impl specs::System<()> for RenderSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        use specs::Join;

        let (map, drawables, lines, positions, cameras, mut targets) = arg.fetch(|world| {
            let map = world.read_resource::<Map>();
            let drawables = world.read::<StaticDrawable>();
            let lines = world.read::<LineDrawable>();
            let positions = world.read::<Position>();
            let cameras = world.write::<super::camera::Camera>();
            let targets = world.write::<DrawableRender>();
            (map, drawables, lines, positions, cameras, targets)
        });

        for target in (&mut targets).iter() {
            target.overlay.clear();
        }

        for line in (&lines).iter() {
            for (camera, target) in (&cameras, &mut targets).iter() {
                let points = bresenham(line.start, line.end);
                for coord in points {
                    if let Some(point) = coord.relative_to(&camera) {
                        let mut tc: TermCell = ' '.into();
                        tc.bg = Some(if map.occupable(coord.x, coord.y) {
                            ColorValue::Magenta
                        } else {
                            ColorValue::Red
                        });
                        target.overlay.put_at(point, tc);
                    }
                }
            }
        }

        for (drawable, position) in (&drawables, &positions).iter() {
            for (camera, target) in (&cameras, &mut targets).iter() {
                if let Some(point) = position.relative_to(&camera) {
                    target.overlay.blend_at(point, drawable.tc);
                }
            }
        }
    }
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
