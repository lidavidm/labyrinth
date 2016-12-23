use std::collections::VecDeque;

use rand::{self, Rng};
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
    modified_cells: VecDeque<(usize, MapCell)>,
    actual_map: Vec<MapCell>,
}

pub struct RenderSystem {
}

pub struct BuilderSystem {
}

impl Map {
    pub fn new(window: Window) -> Map {
        Map {
            map: vec![MapCell::Null; (window.width * window.height) as usize],
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
            modified_cells: VecDeque::new(),
            actual_map: Vec::new(),
        }
    }

    pub fn dig_feature(&mut self, map: &mut Map) {
        use self::MapCell::*;

        if self.num_iterations == 0 {
            self.actual_map.clone_from(&map.map);
            for y_offset in (-2)..3 {
                for x_offset in (-2)..3 {
                    let x = (map.window.width as i32 / 2) + x_offset;
                    let y = (map.window.height as i32 / 2) + y_offset;
                    let offset = (y as usize) * (map.window.width as usize) + (x as usize);
                    let cell = if y_offset == -2 || y_offset == 2 || x_offset == -2 || x_offset == 2 {
                        Wall
                    }
                    else {
                        Floor
                    };
                    self.modified_cells.push_back((offset, cell));
                    self.actual_map[offset] = cell;
                }
            }
        }
        else {
            for _ in 0..1000 {
                let index = rand::thread_rng().gen_range(0, self.actual_map.len());
                if let Wall = self.actual_map[index] {
                    let above = util::above(&self.actual_map, map.window.width, index);
                    let below = util::below(&self.actual_map, map.window.width, index);
                    let left = util::left(&self.actual_map, map.window.width, index);
                    let right = util::right(&self.actual_map, map.window.width, index);

                    self.modified_cells.push_back((index, Floor));
                    self.actual_map[index] = Floor;

                    match (above, below, left, right) {
                        (Floor, _, Wall, Wall) => {

                        }

                        (_, Floor, Wall, Wall) => {
                        }

                        (Wall, Wall, Floor, _) => {

                        }

                        (Wall, Wall, _, Floor) => {
                        }

                        _ => continue,
                    }

                    break;
                }
            }
        }
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
            else if map_builder.modified_cells.len() == 0 {
                to_remove.push(entity);
            }

            for _ in 0..2 {
                if let Some((index, cell)) = map_builder.modified_cells.pop_front() {
                    map.map[index] = cell;
                }
            }
        }

        for entity in to_remove {
            builders.remove(entity);
        }
    }
}

mod util {
    use super::MapCell;

    pub fn above(map: &[MapCell], width: u16, index: usize) -> MapCell {
        if index >= width as usize {
            map[index - width as usize]
        }
        else {
            MapCell::Wall
        }
    }

    pub fn below(map: &[MapCell], width: u16, index: usize) -> MapCell {
        let res = index + (width as usize);
        if res < map.len() {
            map[res]
        }
        else {
            MapCell::Wall
        }
    }

    pub fn left(map: &[MapCell], width: u16, index: usize) -> MapCell {
        if index >= 1 {
            let res = index - 1;
            if res / (width as usize) == index / (width as usize) {
                map[res]
            }
            else {
                MapCell::Wall
            }
        }
        else {
            MapCell::Wall
        }
    }

    pub fn right(map: &[MapCell], width: u16, index: usize) -> MapCell {
        let res = index + 1;
        if res < map.len() && res / (width as usize) == index / (width as usize) {
            map[res]
        }
        else {
            MapCell::Wall
        }
    }
}
