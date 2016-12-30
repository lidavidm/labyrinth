use std::sync::mpsc;

use rand::{self, Rng};
use specs;
use termion::event::Key;
use voodoo::window::{Point, Window};

use ::systems::ui;
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
    message_queue: mpsc::Sender<String>,
    ai_begin: mpsc::Sender<()>,
    ai_end: mpsc::Receiver<()>,
    ai_turn: bool,
    state: State,
}

#[derive(Clone,Copy)]
enum State {
    Toplevel,
    Targeting,
}

impl InputSystem {
    pub fn new(message_queue: mpsc::Sender<String>, ai_begin: mpsc::Sender<()>, ai_end: mpsc::Receiver<()>) -> (InputSystem, mpsc::Sender<Key>) {
        let (tx, rx) = mpsc::channel();
        (InputSystem {
            inputs: rx,
            message_queue: message_queue,
            ai_begin: ai_begin,
            ai_end: ai_end,
            ai_turn: false,
            state: State::Toplevel,
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

    fn process_movement<'a, OM, I>(&self, direction: Direction, map: &mut Map, movables: I) -> bool
        where OM: 'a + OffsetMovable,
              I: Iterator<Item=(&'a Movable, &'a mut OM)> {
        let mut moved = false;
        let offset = direction.offset();

        for (_, movable) in movables {
            let _ = movable.move_by(offset, map);
            moved = true;
        }

        moved
    }

    fn render(&self, window: &mut Window) {
        use self::State::*;

        match self.state {
            Toplevel => {
                window.clear();
                window.print_at(Point::new(0, 0), "WASD—Move");
                window.print_at(Point::new(0, 1), "   1—Examine");
                window.print_at(Point::new(0, 2), "   2—Interact");
                window.print_at(Point::new(0, 3), "   3—Fire");

                window.print_at(Point::new(14, 0), "I—Inventory");
                window.print_at(Point::new(14, 1), "B—Build");
                window.print_at(Point::new(14, 2), "T—Rest");
                window.print_at(Point::new(14, 3), "F—Journal");
            }

            Targeting => {
                window.clear();
                window.print_at(Point::new(0, 0), "    q—Cancel");
                window.print_at(Point::new(0, 1), " WASD—Manual Aim");
                window.print_at(Point::new(0, 2), "  Tab—Cycle Target");
                window.print_at(Point::new(0, 3), "Space—Confirm Fire");
            }
        }
    }
}

impl specs::System<()> for InputSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        use specs::Join;
        use self::State::*;

        if self.ai_turn {
            if let Ok(()) = self.ai_end.try_recv() {
                self.ai_turn = false;
            }
            else {
                // Required to make specs not panic
                arg.fetch(|_| {});
                return;
            }
        }

        match self.state {
            Toplevel => {
                let (mut res, mut map, mut cameras, focused, mut movables, mut positions, mut lines) = arg.fetch(|world| {
                    (
                        world.write_resource::<ui::CommandPanelResource>(),
                        world.write_resource::<super::map::Map>(),
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

                            if self.process_movement(dir, &mut map, (&movables, &mut positions).iter()) {
                                self.ai_begin.send(()).unwrap();
                                self.ai_turn = true;
                                break;
                            }
                        }

                        Key::Char('3') => {
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
                            self.state = Targeting;
                            break;
                        }

                        _ => {}
                    }
                }

                self.render(&mut res.window);
            }

            Targeting => {
                let (
                    mut res, mut map, entities,
                    mut cameras, focused, mut movables,
                    positions, mut lines, healths,
                    mut attacked, equipped,
                ) = arg.fetch(|world| {
                    (
                        world.write_resource::<ui::CommandPanelResource>(),
                        world.write_resource::<super::map::Map>(),
                        world.entities(),
                        world.write::<super::camera::Camera>(),
                        world.read::<super::ui::Focus>(),
                        world.write::<Movable>(),
                        world.write::<Position>(),
                        world.write::<super::drawable::LineDrawable>(),
                        world.write::<super::health::Health>(),
                        world.write::<super::combat::Attack>(),
                        world.read::<super::player::Equip>(),
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
                            self.process_movement(dir, &mut map, (&movables, &mut lines).iter());
                        }

                        Key::Char('3') | Key::Char(' ') => {
                            let mut points = None;
                            for (entity, line, _) in (&entities, &lines, &movables).iter() {
                                points = Some(super::drawable::bresenham(line.start, line.end));
                                arg.delete(entity);
                            }
                            for (entity, _) in (&entities, &focused).iter() {
                                movables.insert(entity, Movable);
                            }
                            self.state = Toplevel;

                            if key == Key::Char(' ') {
                                self.ai_begin.send(()).unwrap();
                                self.ai_turn = true;

                                let mut attack = None;
                                for (entity, _, equip) in (&entities, &focused, &equipped).iter() {
                                    if let Some(super::player::Item {
                                        kind: super::player::ItemKind::Weapon {
                                            damage, accuracy
                                        },
                                        ..
                                    }) = equip.left_hand {
                                        if rand::thread_rng().gen_range(0, 1000) < accuracy {
                                            attack = Some(super::combat::Attack {
                                                damage: damage,
                                                accuracy: accuracy,
                                                source: entity,
                                            });
                                        }
                                        else {
                                            self.message_queue.send("You missed!".into()).unwrap();
                                        }
                                        break;
                                    }
                                }

                                if let Some(attack) = attack {
                                    let mut hit = false;
                                    for target in points.unwrap().iter().skip(1) {
                                        if !map.passable(target.x, target.y) {
                                            for (entity, pos, _health) in (&entities, &positions, &healths).iter() {
                                                if pos.x == target.x && pos.y == target.y {
                                                    self.message_queue.send(format!("Targeted {}, {}", target.x, target.y)).unwrap();
                                                    hit = true;
                                                    attacked.insert(entity, attack);
                                                }
                                            }

                                            if !hit {
                                                hit = true;
                                                self.message_queue.send("You hit a wall.".into()).unwrap();
                                            }

                                            break;
                                        }
                                    }

                                    if !hit {
                                        self.message_queue.send("You hit nothing.".into()).unwrap();
                                    }
                                }

                                break;
                            }

                            break;
                        }

                        _ => {}
                    }
                }

                self.render(&mut res.window);
            }
        }
    }
}
