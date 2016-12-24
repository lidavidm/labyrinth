use std::sync::mpsc;

use specs;
use termion::event::Key;

pub struct Movable;

impl specs::Component for Movable {
    type Storage = specs::VecStorage<Movable>;
}

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

        let (map, mut cameras, movables, mut positions) = arg.fetch(|world| {
            (world.read_resource::<super::map::Map>(),
             world.write::<super::camera::Camera>(),
             world.read::<Movable>(),
             world.write::<super::position::Position>())
        });

        for key in self.inputs.try_iter() {
            match key {
                Key::Up | Key::Down | Key::Left | Key::Right => {
                    let offset = match key {
                        Key::Up => (0, -1),
                        Key::Down => (0, 1),
                        Key::Left => (-1, 0),
                        Key::Right => (1, 0),
                        _ => unreachable!(),
                    };

                    for camera in (&mut cameras).iter() {
                        let new_x = camera.position.x as i32 + offset.0;
                        let new_y = camera.position.y as i32 + offset.1;

                        let new_x = if new_x < 0 { 0 } else { new_x as u16 };
                        let new_y = if new_y < 0 { 0 } else { new_y as u16 };

                        let new_x = ::std::cmp::min(new_x, camera.max_x());
                        let new_y = ::std::cmp::min(new_y, camera.max_y());

                        camera.position.x = new_x;
                        camera.position.y = new_y;
                    }
                }

                Key::Char('w') | Key::Char('a') | Key::Char('s') | Key::Char('d') => {
                    let offset = match key {
                        Key::Char('w') => (0, -1),
                        Key::Char('s') => (0, 1),
                        Key::Char('a') => (-1, 0),
                        Key::Char('d') => (1, 0),
                        _ => unreachable!(),
                    };

                    for (_, position) in (&movables, &mut positions).iter() {
                        // TODO: factor out common movement code?
                        let new_x = position.x as i32 + offset.0;
                        let new_y = position.y as i32 + offset.1;

                        let new_x = if new_x < 0 { 0 } else { new_x as usize };
                        let new_y = if new_y < 0 { 0 } else { new_y as usize };

                        // let new_x = ::std::cmp::min(new_x, camera.max_x());
                        // let new_y = ::std::cmp::min(new_y, camera.max_y());

                        position.x = new_x;
                        position.y = new_y;
                    }
                }

                _ => continue,
            }
        }
    }
}
