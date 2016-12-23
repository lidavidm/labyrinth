use specs;
use voodoo::compositor::Compositor;
use voodoo::window::{Point, Window};

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum MapCell {
    Null,
    Wall,
    Floor,
}

pub struct Map {
    pub map: Vec<MapCell>,
    window: Window,
}

pub struct MapBuilder {
    pub num_iterations: usize,
}

pub struct RenderSystem {
}

pub struct BuilderSystem {
}

impl Map {
    pub fn new(window: Window) -> Map {
        Map {
            map: vec![MapCell::Floor; (window.width * window.height) as usize],
            window: window,
        }
    }

    pub fn render(&mut self) {
        use self::MapCell::*;
        for (offset, cell) in self.map.iter().enumerate() {
            let x = offset as u16 % self.window.width;
            let y = offset as u16 / self.window.width;
            self.window.put_at(
                Point::new(x, y),
                match *cell {
                    Null => ' ',
                    Wall => '#',
                    Floor => '·',
                }
            );
        }
    }

    pub fn refresh(&self, compositor: &mut Compositor) {
        self.window.refresh(compositor);
    }
}

impl specs::Component for Map {
    type Storage = specs::VecStorage<Map>;
}

impl MapBuilder {
    pub fn new() -> MapBuilder {
        MapBuilder {
            num_iterations: 0,
        }
    }

    pub fn dig_feature(&mut self, map: &mut Map) {
        map.map[100 + self.num_iterations] = MapCell::Null;
        self.num_iterations += 1;
    }
}

impl specs::Component for MapBuilder {
    type Storage = specs::VecStorage<MapBuilder>;
}

impl RenderSystem {
    pub fn new() -> RenderSystem {
        RenderSystem {
        }
    }
}

impl specs::System<()> for RenderSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        use specs::Join;

        let maps = &mut arg.fetch(|world| world.write::<Map>());

        for ref mut map in maps.iter() {
            map.render();
        }
    }
}

impl BuilderSystem {
    pub fn new() -> BuilderSystem {
        BuilderSystem {
        }
    }
}

impl specs::System<()> for BuilderSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        use specs::Join;

        let (entities, mut maps, mut builders) = arg.fetch(|world| {
            (world.entities(), world.write::<Map>(), world.write::<MapBuilder>())
        });

        let mut to_remove = vec![];
        for (entity, mut map, mut map_builder) in (&entities, &mut maps, &mut builders).iter() {
            if map_builder.num_iterations < 10 {
                map_builder.dig_feature(map);
            }
            else {
                to_remove.push(entity);
            }
        }

        for entity in to_remove {
            builders.remove(entity);
        }
    }
}
