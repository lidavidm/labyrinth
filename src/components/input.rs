use std::sync::mpsc;

use specs;
use termion::event::Key;
use voodoo::window::Point;

use ::util::Direction;
use super::map::Map;
use super::position::Position;

pub struct Movable;

impl specs::Component for Movable {
    type Storage = specs::VecStorage<Movable>;
}

pub trait OffsetMovable {
    fn move_by(&mut self, offset: (i32, i32), map: &mut Map) -> Result<(), ()>;
}

pub struct InputSystem {
    pub inputs: mpsc::Receiver<Key>,
    panel_state: PanelState,
}

#[derive(Clone,Copy)]
enum PanelState {
    Toplevel,
    Targeting,
}

impl InputSystem {
    pub fn new() -> (InputSystem, mpsc::Sender<Key>) {
        let (tx, rx) = mpsc::channel();
        (InputSystem {
            inputs: rx,
            panel_state: PanelState::Toplevel,
        }, tx)
    }

    fn process_panning<'a, I: Iterator<Item=&'a mut super::camera::Camera>>(&self, direction: Direction, cameras: I) {
        let offset = direction.offset();
        for camera in cameras {
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

    fn process_movement<'a, OM, I>(&self, direction: Direction, map: &mut Map, movables: I)
        where OM: 'a + OffsetMovable,
              I: Iterator<Item=(&'a Movable, &'a mut OM)> {
        let offset = direction.offset();

        for (_, movable) in movables {
            let _ = movable.move_by(offset, map);
        }
    }
}

impl specs::System<()> for InputSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        use specs::Join;
        use self::PanelState::*;

        let (mut res, mut map, entities, mut cameras, focused, mut movables, mut positions, mut lines) = arg.fetch(|world| {
            (
                world.write_resource::<super::ui::CommandPanelResource>(),
                world.write_resource::<super::map::Map>(),
                world.entities(),
                world.write::<super::camera::Camera>(),
                world.read::<super::ui::Focus>(),
                world.write::<Movable>(),
                world.write::<Position>(),
                world.write::<super::drawable::LineDrawable>(),
            )
        });

        for key in self.inputs.try_iter() {
            match key {
                Key::Up | Key::Down | Key::Left | Key::Right => {
                    let dir = match key {
                        Key::Up => Direction::Up,
                        Key::Down => Direction::Down,
                        Key::Left => Direction::Left,
                        Key::Right => Direction::Right,
                        _ => unreachable!(),
                    };
                    self.process_panning(dir, (&mut cameras).iter());
                }

                Key::Char('w') | Key::Char('a') | Key::Char('s') | Key::Char('d') => {
                    let dir = match key {
                        Key::Char('w') => Direction::Up,
                        Key::Char('s') => Direction::Down,
                        Key::Char('a') => Direction::Left,
                        Key::Char('d') => Direction::Right,
                        _ => unreachable!(),
                    };

                    self.process_movement(dir, &mut map, (&movables, &mut positions).iter());
                    self.process_movement(dir, &mut map, (&movables, &mut lines).iter());
                }

                Key::Char('1') | Key::Char('2') | Key::Char('3') |
                Key::Char('q') => {
                    self.panel_state = match (self.panel_state, key) {
                        (Toplevel, Key::Char('3')) => {
                            movables.clear();

                            let mut start_pos = Position { x: 0, y: 0 };
                            for (_, pos) in (&focused, &positions).iter() {
                                start_pos.x = pos.x;
                                start_pos.y = pos.y;
                            }

                            let e = arg.create();
                            lines.insert(e, super::drawable::LineDrawable {
                                start: start_pos,
                                end: start_pos,
                            });
                            movables.insert(e, Movable);
                            Targeting
                        },
                        (Targeting, Key::Char('q')) => {
                            // TODO: need to delete entities
                            movables.clear();
                            Toplevel
                        },
                        _ => Toplevel,
                    };
                }

                Key::Char('I') | Key::Char('B') | Key::Char('T') |
                Key::Char('F') => {
                }

                _ => continue,
            }
        }

        match self.panel_state {
            Toplevel => {
                res.window.clear();
                res.window.print_at(Point::new(0, 0), "WASD—Move");
                res.window.print_at(Point::new(0, 1), "   1—Examine");
                res.window.print_at(Point::new(0, 2), "   2—Interact");
                res.window.print_at(Point::new(0, 3), "   3—Fire");

                res.window.print_at(Point::new(14, 0), "I—Inventory");
                res.window.print_at(Point::new(14, 1), "B—Build");
                res.window.print_at(Point::new(14, 2), "T—Rest");
                res.window.print_at(Point::new(14, 3), "F—Journal");
            }

            Targeting => {
                res.window.clear();
                res.window.print_at(Point::new(0, 0), "    q—Cancel");
                res.window.print_at(Point::new(0, 1), " WASD—Manual Aim");
                res.window.print_at(Point::new(0, 2), "  Tab—Cycle Target");
                res.window.print_at(Point::new(0, 3), "Space—Confirm Fire");
            }
        }
    }
}
