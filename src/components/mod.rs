use specs::World;

pub mod camera;
pub mod drawable;
pub mod input;
pub mod map;
pub mod position;

pub fn register_all(world: &mut World) {
    world.register::<camera::Camera>();

    world.register::<drawable::StaticDrawable>();
    world.register::<drawable::DrawableRender>();

    world.register::<map::MapRender>();
    world.register::<map::MapBuilder>();

    world.register::<position::Position>();
}
