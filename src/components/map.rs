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
            for y_offset in (-3)..4 {
                for x_offset in (-3)..4 {
                    let x = (map.window.width as i32 / 2) + x_offset;
                    let y = (map.window.height as i32 / 2) + y_offset;
                    let offset = (y as usize) * (map.window.width as usize) + (x as usize);
                    let cell = if y_offset == -3 || y_offset == 3 || x_offset == -3 || x_offset == 3 {
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
            'testing: for _ in 0..1000 {
                let index = rand::thread_rng().gen_range(0, self.actual_map.len());
                if let Wall = self.actual_map[index] {
                    let above = util::above(&self.actual_map, map.window.width, index)
                        .and_then(|i| self.actual_map.get(i)).cloned().unwrap_or(Floor);
                    let below = util::below(&self.actual_map, map.window.width, index)
                        .and_then(|i| self.actual_map.get(i)).cloned().unwrap_or(Floor);
                    let left = util::left(&self.actual_map, map.window.width, index)
                        .and_then(|i| self.actual_map.get(i)).cloned().unwrap_or(Floor);
                    let right = util::right(&self.actual_map, map.window.width, index)
                        .and_then(|i| self.actual_map.get(i)).cloned().unwrap_or(Floor);

                    enum Direction {
                        Up,
                        Down,
                        Left,
                        Right,
                    }

                    let direction = match (above, below, left, right) {
                        (Floor, _, Wall, Wall) => {
                            Direction::Down
                        }

                        (_, Floor, Wall, Wall) => {
                            Direction::Up
                        }

                        (Wall, Wall, Floor, _) => {
                            Direction::Right
                        }

                        (Wall, Wall, _, Floor) => {
                            Direction::Left
                        }

                        _ => continue 'testing,
                    };

                    if rand::thread_rng().next_f32() < 0.6 {
                        // Corridor
                        let width = rand::thread_rng().gen_range(1, 3);
                        let length = rand::thread_rng().gen_range(6, 20);
                        let mut cells = vec![];

                        let mut cur = index;
                        for _ in 0..length {
                            let new_index = match direction {
                                Direction::Down => {
                                    util::below(&self.actual_map, map.window.width, cur)
                                }
                                Direction::Up => {
                                    util::above(&self.actual_map, map.window.width, cur)
                                }
                                Direction::Right => {
                                    util::right(&self.actual_map, map.window.width, cur)
                                }
                                Direction::Left => {
                                    util::left(&self.actual_map, map.window.width, cur)
                                }
                            }.and_then(|idx| if self.actual_map[idx] != Null { None } else { Some(idx) });

                            if let Some(idx) = new_index {
                                cells.push((idx, Floor));
                                cur = idx;
                                // TODO: check nullness
                                if let Some(idx) = match direction {
                                    Direction::Down | Direction::Up => {
                                        util::left(&self.actual_map, map.window.width, cur)
                                    }
                                    Direction::Right | Direction::Left => {
                                        util::above(&self.actual_map, map.window.width, cur)
                                    }
                                } {
                                    cells.push((idx, Wall));
                                }
                                if let Some(idx) = match direction {
                                    Direction::Down | Direction::Up => {
                                        util::right(&self.actual_map, map.window.width, cur)
                                    }
                                    Direction::Right | Direction::Left => {
                                        util::below(&self.actual_map, map.window.width, cur)
                                    }
                                } {
                                    cells.push((idx, Wall));
                                }
                            }
                            else {
                                continue 'testing;
                            }
                        }

                        self.actual_map[index] = Floor;
                        self.modified_cells.push_back((index, Floor));
                        for &(idx, c) in cells.iter() {
                            self.actual_map[idx] = c;
                        }
                        self.modified_cells.extend(&cells);
                        self.actual_map[cur] = Wall;
                        self.modified_cells.push_back((cur, Wall));
                    }
                    else {
                        // Room
                        let width = rand::thread_rng().gen_range(6, 15);
                        let height = rand::thread_rng().gen_range(6, 15);

                        let offset = rand::thread_rng().gen_range(1, width - 1);

                        let mut cells = vec![];

                        let mut cur = index;

                        for r in 0..height {
                            // TODO: refactor this
                            let new_index = match direction {
                                Direction::Down => {
                                    util::below(&self.actual_map, map.window.width, cur)
                                }
                                Direction::Up => {
                                    util::above(&self.actual_map, map.window.width, cur)
                                }
                                Direction::Right => {
                                    util::right(&self.actual_map, map.window.width, cur)
                                }
                                Direction::Left => {
                                    util::left(&self.actual_map, map.window.width, cur)
                                }
                            }.and_then(|idx| if self.actual_map[idx] != Null { None } else { Some(idx) });

                            if let Some(idx) = new_index {
                                cells.push((idx, if r == height - 1 {
                                    Wall
                                } else {
                                    Floor
                                }));
                                cur = idx;

                                let mut side_idx = idx;
                                for off in 0..offset {
                                    if let Some(new_side_idx) = match direction {
                                        Direction::Down | Direction::Up => {
                                            util::left(&self.actual_map, map.window.width, side_idx)
                                        }
                                        Direction::Right | Direction::Left => {
                                            util::above(&self.actual_map, map.window.width, side_idx)
                                        }
                                    }.and_then(|idx| if self.actual_map[idx] != Null { None } else { Some(idx) }) {
                                        cells.push((new_side_idx, if r == 0 || r == height - 1 || off == offset - 1 {
                                            Wall
                                        } else {
                                            Floor
                                        }));
                                        side_idx = new_side_idx;
                                    }
                                    else {
                                        continue 'testing;
                                    }
                                }

                                let mut side_idx = idx;
                                for off in offset+1..width {
                                    if let Some(new_side_idx) = match direction {
                                        Direction::Down | Direction::Up => {
                                            util::right(&self.actual_map, map.window.width, side_idx)
                                        }
                                        Direction::Right | Direction::Left => {
                                            util::below(&self.actual_map, map.window.width, side_idx)
                                        }
                                    }.and_then(|idx| if self.actual_map[idx] != Null { None } else { Some(idx) }) {
                                        cells.push((new_side_idx, if r == 0 || r == height - 1 || off == width - 1 {
                                            Wall
                                        } else {
                                            Floor
                                        }));
                                        side_idx = new_side_idx;
                                    }
                                    else {
                                        continue 'testing;
                                    }
                                }
                            }
                            else {
                                continue 'testing;
                            }
                        }

                        self.actual_map[index] = Floor;
                        self.modified_cells.push_back((index, Floor));
                        for &(idx, c) in cells.iter() {
                            self.actual_map[idx] = c;
                        }
                        self.modified_cells.extend(&cells);
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
            if map_builder.num_iterations < 30 {
                map_builder.dig_feature(map);
            }
            else if map_builder.modified_cells.len() == 0 {
                to_remove.push(entity);
                // for cell in map.map.iter_mut() {
                //     if let MapCell::Null = *cell {
                //         *cell = MapCell::Wall;
                //     }
                // }
            }

            if let Some((index, cell)) = map_builder.modified_cells.pop_front() {
                map.map[index] = cell;
            }
        }

        for entity in to_remove {
            builders.remove(entity);
        }
    }
}

mod util {
    use super::MapCell;

    pub fn above(map: &[MapCell], width: u16, index: usize) -> Option<usize> {
        if index >= width as usize {
            Some(index - width as usize)
        }
        else {
            None
        }
    }

    pub fn below(map: &[MapCell], width: u16, index: usize) -> Option<usize> {
        let res = index + (width as usize);
        if res < map.len() {
            Some(res)
        }
        else {
            None
        }
    }

    pub fn left(map: &[MapCell], width: u16, index: usize) -> Option<usize> {
        if index >= 1 {
            let res = index - 1;
            if res / (width as usize) == index / (width as usize) {
                Some(res)
            }
            else {
                None
            }
        }
        else {
            None
        }
    }

    pub fn right(map: &[MapCell], width: u16, index: usize) -> Option<usize> {
        let res = index + 1;
        if res < map.len() && res / (width as usize) == index / (width as usize) {
            Some(res)
        }
        else {
            None
        }
    }
}
