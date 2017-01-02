use std::cell::Cell;
use std::sync::mpsc;

use specs;
use termion::event::{Key, MouseEvent};
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
    pub inputs: mpsc::Receiver<Event>,
    message_queue: mpsc::Sender<String>,
    ai_begin: mpsc::Sender<()>,
    ai_end: mpsc::Receiver<()>,
    ai_turn: Cell<bool>,
    state: State,
    transitions: ::screen::TransitionChannel,
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Event {
    Key(Key),
    MouseDown(Point),
    MouseHover(Point),
    MouseRelease(Point),
}

#[derive(Clone,Copy)]
enum State {
    Toplevel,
    Targeting,
}

impl InputSystem {
    pub fn new(transitions: ::screen::TransitionChannel, message_queue: mpsc::Sender<String>, ai_begin: mpsc::Sender<()>, ai_end: mpsc::Receiver<()>) -> (InputSystem, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel();
        (InputSystem {
            inputs: rx,
            message_queue: message_queue,
            ai_begin: ai_begin,
            ai_end: ai_end,
            ai_turn: Cell::new(false),
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

    fn process_movement<'a, OM, I>(&self, direction: Direction, map: &mut Map, movables: I)
        where OM: 'a + OffsetMovable,
              I: Iterator<Item=(&'a Movable, &'a mut OM)> {
        let offset = direction.offset();

        for (_, movable) in movables {
            let _ = movable.move_by(offset, map);
        }
    }

    fn end_turn(&self) {
        self.ai_begin.send(()).unwrap();
        self.ai_turn.set(false);
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

                window.print_at(Point::new(14, 0), "4—Melee");
                window.print_at(Point::new(14, 1), "5—Item");

                window.print_at(Point::new(22, 0), "I—Inventory");
                window.print_at(Point::new(22, 1), "B—Build");
                window.print_at(Point::new(22, 2), "T—Rest");
                window.print_at(Point::new(22, 3), "F—Journal");
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

        if self.ai_turn.get() {
            if let Ok(()) = self.ai_end.try_recv() {
                self.ai_turn.set(false);
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
                for event in self.inputs.try_iter() {
                    match event {
                        Event::Key(Key::Esc) => self.transitions.send(::screen::StateTransition::Quit).unwrap(),

                        Event::Key(Key::Up) => self.process_panning(Direction::Up, (&mut cameras).iter()),
                        Event::Key(Key::Down) => self.process_panning(Direction::Down, (&mut cameras).iter()),
                        Event::Key(Key::Left) => self.process_panning(Direction::Left, (&mut cameras).iter()),
                        Event::Key(Key::Right) => self.process_panning(Direction::Right, (&mut cameras).iter()),

                        Event::Key(Key::Char('w')) => {
                            self.process_movement(Direction::Up, &mut map,
                                                  (&movables, &mut positions).iter());
                            self.end_turn();
                        }
                        Event::Key(Key::Char('s')) => {
                            self.process_movement(Direction::Down, &mut map,
                                                  (&movables, &mut positions).iter());
                            self.end_turn();
                        }
                        Event::Key(Key::Char('a')) => {
                            self.process_movement(Direction::Left, &mut map,
                                                  (&movables, &mut positions).iter());
                            self.end_turn();
                        }
                        Event::Key(Key::Char('d')) => {
                            self.process_movement(Direction::Right, &mut map,
                                                  (&movables, &mut positions).iter());
                            self.end_turn();
                        }

                        Event::Key(Key::Char('3')) => {
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
                    mut lines, covers, healths,
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
                        world.read::<super::health::Cover>(),
                        world.read::<super::health::Health>(),
                        world.write::<super::combat::Attack>(),
                        world.read::<super::player::Equip>(),
                    )
                });

                for event in self.inputs.try_iter() {
                    match event {
                        Event::Key(Key::Up) => self.process_panning(Direction::Up, (&mut cameras).iter()),
                        Event::Key(Key::Down) => self.process_panning(Direction::Down, (&mut cameras).iter()),
                        Event::Key(Key::Left) => self.process_panning(Direction::Left, (&mut cameras).iter()),
                        Event::Key(Key::Right) => self.process_panning(Direction::Right, (&mut cameras).iter()),

                        Event::Key(Key::Char('w')) =>
                            self.process_movement(Direction::Up, &mut map,
                                                  (&movables, &mut lines).iter()),
                        Event::Key(Key::Char('s')) =>
                            self.process_movement(Direction::Down, &mut map,
                                                  (&movables, &mut lines).iter()),
                        Event::Key(Key::Char('a')) =>
                            self.process_movement(Direction::Left, &mut map,
                                                  (&movables, &mut lines).iter()),
                        Event::Key(Key::Char('d')) =>
                            self.process_movement(Direction::Right, &mut map,
                                                  (&movables, &mut lines).iter()),

                        Event::Key(Key::Esc) | Event::Key(Key::Char('3')) |
                        Event::Key(Key::Char('q')) | Event::Key(Key::Char(' ')) => {
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

                            if event == Event::Key(Key::Char(' ')) {
                                self.end_turn();

                                if let (Some((start, end)), Some((attacker, equip))) = (points, attacker) {
                                    match ::util::combat::resolve(&map, attacker, &equip, start, end, &healths, &covers) {
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
