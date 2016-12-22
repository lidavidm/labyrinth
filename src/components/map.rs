use specs;

pub enum MapCell {
    Null,
    Wall,
    Floor,
}

pub struct Map;

impl Map {
    pub fn new() -> Map {
        Map
    }
}

impl specs::Component for Map {
    type Storage = specs::VecStorage<Map>;
}

pub struct RenderSystem;

impl RenderSystem {
    pub fn new() -> RenderSystem {
        RenderSystem
    }
}

impl specs::System<()> for RenderSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        use specs::Join;

        let maps = arg.fetch(|world| world.read::<Map>());

        for map in maps.iter() {

        }
    }
}
