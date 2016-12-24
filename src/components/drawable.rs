use specs;
use voodoo;
use voodoo::overlay::Overlay;
use voodoo::window::TermCell;

pub struct StaticDrawable {
    pub tc: TermCell,
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

        let (drawables, positions, cameras, mut targets) = arg.fetch(|world| {
            let drawables = world.read::<StaticDrawable>();
            let positions = world.read::<super::position::Position>();
            let cameras = world.write::<super::camera::Camera>();
            let targets = world.write::<DrawableRender>();
            (drawables, positions, cameras, targets)
        });

        for target in (&mut targets).iter() {
            target.overlay.clear();
        }

        for (drawable, position) in (&drawables, &positions).iter() {
            for (camera, target) in (&cameras, &mut targets).iter() {
                if let Some(point) = position.relative_to(&camera) {
                    target.overlay.put_at(point, drawable.tc);
                }
            }
        }
    }
}
