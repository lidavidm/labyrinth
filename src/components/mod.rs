use specs::World;

pub mod input;
pub mod map;

pub fn register_all(world: &mut World) {
    world.register::<map::MapRender>();
    world.register::<map::MapBuilder>();
}
