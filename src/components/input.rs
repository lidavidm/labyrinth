use std::sync::mpsc;

use specs;
use termion::event::Key;

pub struct InputSystem {
    pub inputs: mpsc::Receiver<Key>,
}

impl InputSystem {
    pub fn new() -> (InputSystem, mpsc::Sender<Key>) {
        let (tx, rx) = mpsc::channel();
        (InputSystem {
            inputs: rx,
        }, tx)
    }
}

impl specs::System<()> for InputSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        use specs::Join;

        let mut maps = arg.fetch(|world| world.write::<super::map::Map>());

        for key in self.inputs.try_iter() {
            let offset = match key {
                Key::Up => (0, -1),
                Key::Down => (0, 1),
                Key::Left => (-1, 0),
                Key::Right => (1, 0),
                _ => continue,
            };

            for map in (&mut maps).iter() {
                let new_x = map.position.x as i32 + offset.0;
                let new_y = map.position.y as i32 + offset.1;

                let new_x = if new_x < 0 { 0 } else { new_x as u16 };
                let new_y = if new_y < 0 { 0 } else { new_y as u16 };

                let new_x = ::std::cmp::min(new_x, map.max_x());
                let new_y = ::std::cmp::min(new_y, map.max_y());

                map.position.x = new_x;
                map.position.y = new_y;
            }
        }
    }
}
