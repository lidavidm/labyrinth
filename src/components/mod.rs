use specs::World;

pub mod camera;
pub mod drawable;
pub mod health;
pub mod input;
pub mod map;
pub mod position;
pub mod ui;

pub fn register_all(world: &mut World) {
    world.register::<camera::Camera>();

    world.register::<drawable::LineDrawable>();
    world.register::<drawable::StaticDrawable>();
    world.register::<drawable::DrawableRender>();

    world.register::<health::Health>();

    world.register::<input::Movable>();

    world.register::<map::MapRender>();
    world.register::<map::MapBuilder>();

    world.register::<position::Position>();

    world.register::<ui::Focus>();
}
