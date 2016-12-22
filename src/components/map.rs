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
    map: Vec<MapCell>,
    window: Window,
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

pub struct RenderSystem {
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
