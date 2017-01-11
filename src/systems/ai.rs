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

use std::sync::mpsc;

use rand::{self, Rng};
use specs::{self, Join};

use ::components::{ai, combat, drawable, health, map, player, position};
use ::components::input::OffsetMovable;
use ::util;

pub struct AiSystem {
    message_queue: mpsc::Sender<String>,
    ai_begin: mpsc::Receiver<()>,
    ai_end: mpsc::Sender<()>,
}

pub struct DeadSystem {
    transitions: ::screen::TransitionChannel,
}

impl AiSystem {
    pub fn new(message_queue: mpsc::Sender<String>, ai_begin: mpsc::Receiver<()>, ai_end: mpsc::Sender<()>) -> AiSystem {
        AiSystem {
            message_queue: message_queue,
            ai_begin: ai_begin,
            ai_end: ai_end,
        }
    }
}

impl specs::System<()> for AiSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        if let Ok(()) = self.ai_begin.try_recv() {

        }
        else {
            // Required to make specs not panic
            arg.fetch(|_| {});
            return;
        }

        let (mut map, entities, mut chase_behaviors, mut attacked, dead, covers, healths, equipped, players, mut positions) = arg.fetch(|world| {
            (
                world.write_resource::<map::Map>(),
                world.entities(),
                world.write::<ai::ChaseBehavior>(),
                world.write::<combat::Attack>(),
                world.read::<ai::Dead>(),
                world.read::<health::Cover>(),
                world.read::<health::Health>(),
                world.read::<player::Equip>(),
                world.read::<player::Player>(),
                world.write::<position::Position>(),
            )
        });

        let mut player_position = (0, 0);
        for (_, position) in (&players, &positions).iter() {
            player_position = (position.x, position.y);
        }

        for (me, chaser, position, equip, _) in (&entities, &mut chase_behaviors, &mut positions, &equipped, !&dead).iter() {
            if util::distance2(player_position, (position.x, position.y)) < 25 {
                chaser.spotted = Some(player_position);
            }

            if let Some((x, y)) = chaser.spotted {

                if util::distance2((x, y), (position.x, position.y)) < 9 {
                    if let Some(_) = map.contents(x, y) {
                        match ::util::combat::resolve(
                            &map, me, &equip, *position,
                            position::Position::new(player_position.0, player_position.1),
                            false, &healths, &covers) {
                            ::util::combat::CombatResult::NothingEquipped => {
                            }
                            ::util::combat::CombatResult::Miss => {
                                self.message_queue.send("Enemy missed!".into()).unwrap();
                            }
                            ::util::combat::CombatResult::HitNothing => {
                                self.message_queue.send("Enemy hit nothing.".into()).unwrap();
                            }
                            ::util::combat::CombatResult::HitEnvironment => {
                                self.message_queue.send("Enemy hit a wall.".into()).unwrap();
                            }
                            ::util::combat::CombatResult::HitEntity(target, pos, attack) => {
                                self.message_queue.send(format!("Enemy targeted {}, {}", pos.x, pos.y)).unwrap();
                                attacked.insert(target, attack);
                            }
                            ::util::combat::CombatResult::OutOfRange => {
                                self.message_queue.send("Enemy tried a melee weapon out of range.".into()).unwrap();
                            }
                        }
                        continue;
                    }
                }

                let dx = (x as i32 - position.x as i32).signum();
                let dy = (y as i32 - position.y as i32).signum();

                // Move towards the last known player position if
                // seen, else forget
                if dx != 0 {
                    if let Ok(_) = position.move_by((dx, 0), &mut map) {
                        continue;
                    }
                }
                if dy != 0 {
                    if let Ok(_) = position.move_by((0, dy), &mut map) {
                        continue;
                    }
                }

                chaser.spotted = None;
            }
        }

        self.ai_end.send(()).unwrap();
    }
}

impl DeadSystem {
    pub fn new(transitions: ::screen::TransitionChannel) -> DeadSystem {
        DeadSystem {
            transitions: transitions,
        }
    }
}

impl specs::System<()> for DeadSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut map, entities, dead, mut drawables, mut drops_loot, mut grabbables, players, mut positions) = arg.fetch(|world| {
            (
                world.write_resource::<map::Map>(),
                world.entities(),
                world.read::<ai::Dead>(),
                world.write::<drawable::StaticDrawable>(),
                world.write::<player::DropsLoot>(),
                world.write::<player::Grabbable>(),
                world.read::<player::Player>(),
                world.write::<position::Position>(),
            )
        });

        let mut to_create = vec![];

        for (entity, position, _) in (&entities, &positions, &dead).iter() {
            arg.delete(entity);
            map.vacate(position.x, position.y);

            if let Some(_) = players.get(entity) {
                self.transitions.send(::screen::StateTransition::GameOver).unwrap();
            }
            else {
                if let Some(drop_table) = drops_loot.get(entity) {
                    if rand::thread_rng().gen_range(0, 1000) < drop_table.chance {
                        if let Some(loot) = rand::thread_rng().choose(&drop_table.items) {
                            let corpse = arg.create();
                            drawables.insert(corpse, drawable::StaticDrawable {
                                tc: '␣'.into(),
                            });
                            to_create.push((corpse, *position));
                            grabbables.insert(corpse, player::Grabbable(loot.clone()));
                            map.fill(corpse, position.x, position.y);
                        }
                    }
                }
            }
        }

        for (entity, position) in to_create {
            positions.insert(entity, position);
        }
    }
}
