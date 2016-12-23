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
    pub width: usize,
    pub height: usize,
    pub position: Point,
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
    pub fn new(width: usize, height: usize, window: Window) -> Map {
        Map {
            map: vec![MapCell::Null; width * height],
            width: width,
            height: height,
            position: Point::new(0, 0),
            window: window,
        }
    }

    pub fn render(&mut self) {
        use self::MapCell::*;

        for row_offset in 0..self.window.height {
            let start = (row_offset + self.position.y) as usize * self.width;
            for col_offset in 0..self.window.width {
                let offset = self.position.x as usize + start + col_offset as usize;
                let y = row_offset;
                let x = col_offset;
                self.window.put_at(
                    Point::new(x, y),
                    match self.map[offset] {
                        Null => ' ',
                        Wall => '#',
                        Floor => '·',
                    }
                )
            }
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
                    let x = (map.width as i32 / 2) + x_offset;
                    let y = (map.height as i32 / 2) + y_offset;
                    let offset = (y as usize) * map.width + (x as usize);
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
            use self::util::Direction;
            'testing: for _ in 0..1000 {
                let index = rand::thread_rng().gen_range(0, self.actual_map.len());
                if let Wall = self.actual_map[index] {
                    let above = util::above(&self.actual_map, map.width, index)
                        .and_then(|i| self.actual_map.get(i)).cloned().unwrap_or(Floor);
                    let below = util::below(&self.actual_map, map.width, index)
                        .and_then(|i| self.actual_map.get(i)).cloned().unwrap_or(Floor);
                    let left = util::left(&self.actual_map, map.width, index)
                        .and_then(|i| self.actual_map.get(i)).cloned().unwrap_or(Floor);
                    let right = util::right(&self.actual_map, map.width, index)
                        .and_then(|i| self.actual_map.get(i)).cloned().unwrap_or(Floor);

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

                    let (width_range, length_range) = if rand::thread_rng().next_f32() < 0.6 {
                        // Corridor
                        ((3, 5), (5, 20))
                    }
                    else {
                        // Room
                        ((5, 20), (5, 20))
                    };

                    let res = util::generate_room(index, direction, &self.actual_map, map.width,
                                                  width_range, length_range);
                    if let Some(cells) = res {
                        for &(index, cell) in cells.iter() {
                            self.actual_map[index] = cell;
                        }
                        self.modified_cells.extend(&cells);
                    }
                    else {
                        continue 'testing;
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
            if map_builder.num_iterations < 50 {
                map_builder.dig_feature(map);
            }
            else if map_builder.modified_cells.len() == 0 {
                to_remove.push(entity);
            }

            for _ in 0..5 {
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

    pub enum Direction {
        Up,
        Down,
        Left,
        Right,
    }

    impl Direction {
        pub fn to_index(&self, map: &[MapCell], width: usize, cur: usize) -> Option<usize> {
            match *self {
                Direction::Down => {
                    below(map, width, cur)
                }
                Direction::Up => {
                    above(map, width, cur)
                }
                Direction::Right => {
                    right(map, width, cur)
                }
                Direction::Left => {
                    left(map, width, cur)
                }
            }
        }
    }

    pub fn generate_room(start_point: usize, direction: Direction, actual_map: &[MapCell], map_width: usize,
                         width_range: (usize, usize), height_range: (usize, usize)) -> Option<Vec<(usize, super::MapCell)>> {
        use rand::{self, Rng};
        use super::MapCell::*;

        let width = rand::thread_rng().gen_range(width_range.0, width_range.1);
        let height = rand::thread_rng().gen_range(height_range.0, height_range.1);

        let offset = rand::thread_rng().gen_range(1, width - 1);

        let mut cells = vec![(start_point, Floor)];

        let mut cur = start_point;

        for r in 0..height {
            let new_index = direction.to_index(actual_map, map_width, cur)
                .and_then(|idx| if actual_map[idx] != Null { None } else { Some(idx) });

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
                            left(&actual_map, map_width, side_idx)
                        }
                        Direction::Right | Direction::Left => {
                            above(&actual_map, map_width, side_idx)
                        }
                    }.and_then(|idx| if actual_map[idx] != Null { None } else { Some(idx) }) {
                        cells.push((new_side_idx, if r == 0 || r == height - 1 || off == offset - 1 {
                            Wall
                        } else {
                            Floor
                        }));
                        side_idx = new_side_idx;
                    }
                    else {
                        return None;
                    }
                }

                let mut side_idx = idx;
                for off in offset+1..width {
                    if let Some(new_side_idx) = match direction {
                        Direction::Down | Direction::Up => {
                            right(&actual_map, map_width, side_idx)
                        }
                        Direction::Right | Direction::Left => {
                            below(&actual_map, map_width, side_idx)
                        }
                    }.and_then(|idx| if actual_map[idx] != Null { None } else { Some(idx) }) {
                        cells.push((new_side_idx, if r == 0 || r == height - 1 || off == width - 1 {
                            Wall
                        } else {
                            Floor
                        }));
                        side_idx = new_side_idx;
                    }
                    else {
                        return None;
                    }
                }
            }
            else {
                return None;
            }
        }

        Some(cells)
    }

    pub fn above(map: &[MapCell], width: usize, index: usize) -> Option<usize> {
        if index >= width {
            Some(index - width)
        }
        else {
            None
        }
    }

    pub fn below(map: &[MapCell], width: usize, index: usize) -> Option<usize> {
        let res = index + width;
        if res < map.len() {
            Some(res)
        }
        else {
            None
        }
    }

    pub fn left(map: &[MapCell], width: usize, index: usize) -> Option<usize> {
        if index >= 1 {
            let res = index - 1;
            if res / width == index / width {
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

    pub fn right(map: &[MapCell], width: usize, index: usize) -> Option<usize> {
        let res = index + 1;
        if res < map.len() && res / width  == index / width {
            Some(res)
        }
        else {
            None
        }
    }
}
