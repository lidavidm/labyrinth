use specs;
use voodoo;
use voodoo::color::ColorValue;
use voodoo::overlay::Overlay;
use voodoo::window::TermCell;

use ::util::bresenham;
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
