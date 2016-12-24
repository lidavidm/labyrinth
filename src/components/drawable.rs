use specs;
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

}

impl specs::Component for DrawableRender {
    type Storage = specs::VecStorage<DrawableRender>;
}

impl RenderSystem {

}

impl specs::System<()> for RenderSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        use specs::Join;

        let (drawables, cameras, mut targets) = arg.fetch(|world| {
            let drawables = world.read::<StaticDrawable>();
            let cameras = world.write::<super::camera::Camera>();
            let targets = world.write::<DrawableRender>();
            (drawables, cameras, targets)
        });

        for drawable in drawables.iter() {
            for (camera, target) in (&cameras, &mut targets).iter() {
            }
        }
    }
}
