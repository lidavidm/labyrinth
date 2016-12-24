use specs::World;

pub mod camera;
pub mod drawable;
pub mod input;
pub mod map;

pub fn register_all(world: &mut World) {
    world.register::<camera::Camera>();

    world.register::<drawable::StaticDrawable>();

    world.register::<map::MapRender>();
    world.register::<map::MapBuilder>();
}
