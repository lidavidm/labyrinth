use std::sync::mpsc;

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
    transitions: ::screen::TransitionChannel,
}

#[derive(Clone,Copy)]
enum State {
    Toplevel,
    Targeting,
}

impl InputSystem {
    pub fn new(transitions: ::screen::TransitionChannel, message_queue: mpsc::Sender<String>, ai_begin: mpsc::Sender<()>, ai_end: mpsc::Receiver<()>) -> (InputSystem, mpsc::Sender<Key>) {
        let (tx, rx) = mpsc::channel();
        (InputSystem {
            inputs: rx,
            message_queue: message_queue,
            ai_begin: ai_begin,
            ai_end: ai_end,
            ai_turn: false,
            state: State::Toplevel,
            transitions: transitions,
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
                window.print_at(Point::new(0, 0), "  Esc—Cancel");
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
                        Key::Esc => self.transitions.send(::screen::StateTransition::Quit).unwrap(),
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
                    mut lines, healths,
                    mut attacked, equipped,
                ) = arg.fetch(|world| {
                    (
                        world.write_resource::<ui::CommandPanelResource>(),
                        world.write_resource::<super::map::Map>(),
                        world.entities(),
                        world.write::<super::camera::Camera>(),
                        world.read::<super::ui::Focus>(),
                        world.write::<Movable>(),
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

                        Key::Esc | Key::Char('3') | Key::Char('q') | Key::Char(' ') => {
                            let mut points = None;
                            let mut attacker = None;
                            for (entity, line, _) in (&entities, &lines, &movables).iter() {
                                points = Some((line.start, line.end));
                                arg.delete(entity);
                            }
                            for (entity, _) in (&entities, &focused).iter() {
                                movables.insert(entity, Movable);
                                attacker = Some((entity, equipped.get(entity).unwrap()));
                            }
                            self.state = Toplevel;

                            if key == Key::Char(' ') {
                                self.ai_begin.send(()).unwrap();
                                self.ai_turn = true;

                                if let (Some((start, end)), Some((attacker, equip))) = (points, attacker) {
                                    match ::util::combat::resolve(&map, attacker, &equip, start, end, &healths) {
                                        ::util::combat::CombatResult::NothingEquipped => {
                                            self.message_queue.send("You have nothing equipped!".into()).unwrap();
                                        }
                                        ::util::combat::CombatResult::Miss => {
                                            self.message_queue.send("You missed!".into()).unwrap();
                                        }
                                        ::util::combat::CombatResult::HitNothing => {
                                            self.message_queue.send("You hit nothing.".into()).unwrap();
                                        }
                                        ::util::combat::CombatResult::HitEnvironment => {
                                            self.message_queue.send("You hit a wall.".into()).unwrap();
                                        }
                                        ::util::combat::CombatResult::HitEntity(target, pos, attack) => {
                                            self.message_queue.send(format!("Targeted {}, {}", pos.x, pos.y)).unwrap();
                                            attacked.insert(target, attack);
                                        }
                                    }
                                }
                                else {
                                    panic!("No attacker/no Equip/no target found? {:?} {:?}", points, attacker);
                                }
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
