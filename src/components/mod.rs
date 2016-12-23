use specs::World;

pub mod input;
pub mod map;

pub fn register_all(world: &mut World) {
    world.register::<map::Map>();
    world.register::<map::MapBuilder>();
}
