// Copyright (C) 2016-2017 David Li

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::cell::Cell;
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
    pub inputs: mpsc::Receiver<Event>,
    message_queue: mpsc::Sender<String>,
    ai_begin: mpsc::Sender<()>,
    ai_end: mpsc::Receiver<()>,
    ai_turn: Cell<bool>,
    state: State,
    transitions: ::screen::TransitionChannel,
    sub_screen: mpsc::Sender<::screen::SubScreenEvent<::screen::game::SubGameScreen>>,
    inv_valid: bool,
    inv_list: ::ui::List<::components::player::Item>,
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
    Examining,
    Targeting(bool),
    Inventory,
}

impl InputSystem {
    pub fn new(inv_list: ::ui::List<::components::player::Item>,
               transitions: ::screen::TransitionChannel,
               sub_screen: mpsc::Sender<::screen::SubScreenEvent<::screen::game::SubGameScreen>>,
               message_queue: mpsc::Sender<String>,
               ai_begin: mpsc::Sender<()>, ai_end: mpsc::Receiver<()>) -> (InputSystem, mpsc::Sender<Event>) {
        let (tx, rx) = mpsc::channel();
        (InputSystem {
            inputs: rx,
            message_queue: message_queue,
            ai_begin: ai_begin,
            ai_end: ai_end,
            ai_turn: Cell::new(false),
            state: State::Toplevel,
            transitions: transitions,
            sub_screen: sub_screen,
            inv_valid: false,
            inv_list: inv_list,
        }, tx)
    }

    fn process_movement<'a, OM, I>(&self, direction: Direction, map: &mut Map, movables: I)
        where OM: 'a + OffsetMovable,
              I: Iterator<Item=(&'a Movable, &'a mut OM)> {
        let offset = direction.offset();

        for (_, movable) in movables {
            let _ = movable.move_by(offset, map);
        }
    }

    fn take_items<'a, I, G>(&self, direction: Direction, map: &mut Map, movables: I, grabbables: G)
        -> Vec<specs::Entity>
        where I: Iterator<Item=(&'a Movable, &'a Position, &'a mut super::player::Inventory)>,
              G: ::util::HasStorage<super::player::Grabbable>, {
        let mut to_delete = Vec::new();
        let offset = direction.offset();

        for (_, position, inventory) in movables {
            let new_x = position.x as i32 + offset.0;
            let new_y = position.y as i32 + offset.1;

            let new_x = if new_x < 0 { 0 } else { new_x as usize };
            let new_y = if new_y < 0 { 0 } else { new_y as usize };

            if let Some(entity) = map.contents(new_x, new_y) {
                if let Some(&super::player::Grabbable(ref item)) = grabbables.get(entity) {
                    inventory.contents.push(item.clone());
                    self.message_queue.send(format!("Picked up {}", item.name)).unwrap();
                    to_delete.push(entity);
                }
            }
        }

        to_delete
    }

    fn end_turn(&self) {
        self.ai_begin.send(()).unwrap();
        self.ai_turn.set(false);
    }

    fn render(&self, window: &mut Window) {
        use self::State::*;
        window.clear();

        match self.state {
            Toplevel => {
                window.print_at(Point::new(0, 0), "WASD—Move");
                window.print_at(Point::new(0, 1), "   1—Examine");
                window.print_at(Point::new(0, 2), "   2—Fire Primary");
                window.print_at(Point::new(0, 3), "   3—Fire Secondary");

                window.print_at(Point::new(20, 0), "I—Inventory");
            }

            Examining => {
                window.print_at(Point::new(0, 0), "  Esc—Cancel");
                window.print_at(Point::new(0, 1), " WASD—Move");
                window.print_at(Point::new(0, 2), "Mouse—Describe");
            }

            Targeting(_) => {
                window.print_at(Point::new(0, 0), "  Esc—Cancel");
                window.print_at(Point::new(0, 1), " WASD—Manual Aim");
                window.print_at(Point::new(0, 2), "Mouse—Choose Target");
                window.print_at(Point::new(0, 3), "Space—Confirm Fire");
            }

            Inventory => {
                window.print_at(Point::new(0, 0), "  Esc—Cancel");
                window.print_at(Point::new(0, 1), "   WS—Scroll");
                window.print_at(Point::new(0, 2), "Space—Select");
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
                let (mut res, mut map, focused, mut movables, mut positions, mut lines, mut inventories, grabbables) = arg.fetch(|world| {
                    (
                        world.write_resource::<ui::CommandPanelResource>(),
                        world.write_resource::<super::map::Map>(),
                        world.read::<super::ui::Focus>(),
                        world.write::<Movable>(),
                        world.write::<Position>(),
                        world.write::<super::drawable::LineDrawable>(),
                        world.write::<super::player::Inventory>(),
                        world.read::<super::player::Grabbable>(),
                    )
                });
                for event in self.inputs.try_iter() {
                    match event {
                        Event::Key(Key::Esc) => self.transitions.send(::screen::StateTransition::Quit).unwrap(),

                        Event::Key(Key::Char('w')) => {
                            for entity in self.take_items(Direction::Up, &mut map,
                                                          (&movables, &positions, &mut inventories).iter(),
                                                          &grabbables) {
                                if let Some(pos) = positions.get(entity) {
                                    map.vacate(pos.x, pos.y);
                                }
                                positions.remove(entity);
                                arg.delete(entity);
                            }
                            self.process_movement(Direction::Up, &mut map,
                                                  (&movables, &mut positions).iter());
                            self.end_turn();
                        }
                        Event::Key(Key::Char('s')) => {
                            for entity in self.take_items(Direction::Down, &mut map,
                                                          (&movables, &positions, &mut inventories).iter(),
                                                          &grabbables) {
                                if let Some(pos) = positions.get(entity) {
                                    map.vacate(pos.x, pos.y);
                                }
                                positions.remove(entity);
                                arg.delete(entity);
                            }

                            self.process_movement(Direction::Down, &mut map,
                                                  (&movables, &mut positions).iter());
                            self.end_turn();
                        }
                        Event::Key(Key::Char('a')) => {
                            for entity in self.take_items(Direction::Left, &mut map,
                                                          (&movables, &positions, &mut inventories).iter(),
                                                          &grabbables) {
                                if let Some(pos) = positions.get(entity) {
                                    map.vacate(pos.x, pos.y);
                                }
                                positions.remove(entity);
                                arg.delete(entity);
                            }

                            self.process_movement(Direction::Left, &mut map,
                                                  (&movables, &mut positions).iter());
                            self.end_turn();
                        }
                        Event::Key(Key::Char('d')) => {
                            for entity in self.take_items(Direction::Right, &mut map,
                                                          (&movables, &positions, &mut inventories).iter(),
                                                          &grabbables) {
                                if let Some(pos) = positions.get(entity) {
                                    map.vacate(pos.x, pos.y);
                                }
                                positions.remove(entity);
                                arg.delete(entity);
                            }

                            self.process_movement(Direction::Right, &mut map,
                                                  (&movables, &mut positions).iter());
                            self.end_turn();
                        }

                        Event::Key(Key::Char('i')) => {
                            self.state = Inventory;
                            self.sub_screen.send(::screen::SubScreenEvent::Push(::screen::game::SubGameScreen::Inventory)).unwrap();
                            break;
                        }

                        Event::Key(Key::Char('1')) => {
                            self.state = Examining;
                            break;
                        }

                        Event::Key(Key::Char(c @ '2')) | Event::Key(Key::Char(c @ '3')) => {
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
                            self.state = Targeting(c == '3');
                            break;
                        }

                        _ => {}
                    }
                }

                self.render(&mut res.window);
            }

            Examining => {
                let (mut res, mut map, cameras, movables, mut positions, cover, health, grabbable) = arg.fetch(|world| {
                    (
                        world.write_resource::<ui::CommandPanelResource>(),
                        world.write_resource::<super::map::Map>(),
                        world.read::<super::camera::Camera>(),
                        world.read::<Movable>(),
                        world.write::<Position>(),
                        world.read::<super::health::Cover>(),
                        world.read::<super::health::Health>(),
                        world.read::<super::player::Grabbable>(),
                    )
                });
                for event in self.inputs.try_iter() {
                    match event {
                        Event::Key(Key::Esc) => self.state = Toplevel,

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

                        Event::MouseRelease(point) => {
                            let camera = cameras.iter().next().unwrap();
                            let x = (camera.position.x + point.x) as usize;
                            let y = (camera.position.y + point.y) as usize;

                            if let Some(entity) = map.contents(x, y) {
                                self.message_queue.send(format!("Position: {}, {}", x, y)).unwrap();
                                if let Some(h) = health.get(entity) {
                                    self.message_queue.send(format!("Health: {}/{}", h.health, h.max_health)).unwrap();
                                }
                                if let Some(_) = cover.get(entity) {
                                    self.message_queue.send("Provides cover".into()).unwrap();
                                }
                                if let Some(g) = grabbable.get(entity) {
                                    use ui::list::ListRenderable;

                                    self.message_queue.send("Dropped item".into()).unwrap();
                                    for line in g.0.render() {
                                        self.message_queue.send(line.into()).unwrap();
                                    }
                                }
                            }
                            else {
                                self.message_queue.send("Nothing here.".into()).unwrap();
                            }
                        }

                        _ => {}
                    }
                }

                self.render(&mut res.window);
            }

            Targeting(is_melee) => {
                let (
                    mut res, mut map, entities,
                    cameras, focused, mut movables,
                    mut lines, covers, healths,
                    mut attacked, equipped,
                ) = arg.fetch(|world| {
                    (
                        world.write_resource::<ui::CommandPanelResource>(),
                        world.write_resource::<super::map::Map>(),
                        world.entities(),
                        world.read::<super::camera::Camera>(),
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

                        Event::MouseHover(point) | Event::MouseRelease(point) => {
                            let camera = cameras.iter().next().unwrap();
                            for (_, movable) in (&movables, &mut lines).iter() {
                                movable.end.x = (camera.position.x + point.x) as usize;
                                movable.end.y = (camera.position.y + point.y) as usize;
                            }
                        }

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
                                    match ::util::combat::resolve(&map, attacker, &equip, start, end, is_melee, &healths, &covers) {
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
                                        ::util::combat::CombatResult::OutOfRange => {
                                            self.message_queue.send("You can't hit that with a melee weapon.".into()).unwrap();
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

            Inventory => {
                let (
                    mut res, mut inv, focused, mut dr, mut health, mut equipped, mut inventory,
                ) = arg.fetch(|world| {
                    (
                        world.write_resource::<ui::CommandPanelResource>(),
                        world.write_resource::<ui::InventoryPanelResource>(),
                        world.read::<super::ui::Focus>(),
                        world.write::<super::combat::DamageReduction>(),
                        world.write::<super::health::Health>(),
                        world.write::<super::player::Equip>(),
                        world.write::<super::player::Inventory>(),
                    )
                });

                if !self.inv_valid {
                    if let Some((_, _, inventory)) = (&focused, &equipped, &inventory).iter().next() {
                        self.inv_list.contents.clone_from(&inventory.contents);
                    }

                    self.inv_valid = true;
                }

                for event in self.inputs.try_iter() {
                    match event {
                        Event::Key(Key::Esc) => {
                            self.sub_screen.send(::screen::SubScreenEvent::Pop).unwrap();
                            self.state = Toplevel;

                            if let Some((_, _, inventory)) = (&focused, &equipped, &mut inventory).iter().next() {
                                inventory.contents.clone_from(&self.inv_list.contents);
                            }

                            self.inv_valid = false;
                            self.inv_list.cursor = 0;
                            self.inv_list.contents.clear();
                        }

                        Event::Key(Key::Char('w')) => self.inv_list.move_cursor_up(),
                        Event::Key(Key::Char('s')) => self.inv_list.move_cursor_down(),

                        Event::Key(Key::Char(' ')) => {
                            let result = if let Some(item) = self.inv_list.get_selected() {
                                if let Some((_, equip, _)) = (&focused, &mut equipped, &inventory).iter().next() {
                                    // TODO: if there's an equipped item, place it in the inventory
                                    let item = equip.equip(item.clone());

                                    // Recompute health boost, damage reduction
                                    if let Some((_, dr, hp)) = (&focused, &mut dr, &mut health).iter().next() {
                                        dr.value = 0;
                                        hp.max_health = hp.base_health;
                                        for item in equip.list_equipped() {
                                            if let super::player::ItemKind::Armor { health, damage_reduction } = item.kind {
                                                dr.value += damage_reduction;
                                                hp.max_health += health;
                                            }
                                        }

                                        if dr.value > 0 {
                                            self.message_queue.send(format!("DR: {}", dr.value)).unwrap();
                                        }
                                    }

                                    Some((item, self.inv_list.cursor))
                                }
                                else {
                                    None
                                }
                            } else { None };

                            if let Some((old_item, idx)) = result {
                                if let Some(item) = old_item {
                                    self.inv_list.contents[idx] = item;
                                }
                                else {
                                    self.inv_list.move_cursor_up();
                                    self.inv_list.contents.remove(idx);
                                }
                            }
                        },

                        _ => {}
                    }
                }

                inv.window.clear();
                self.inv_list.refresh(&mut inv.window);
                self.render(&mut res.window);
            }
        }
    }
}
