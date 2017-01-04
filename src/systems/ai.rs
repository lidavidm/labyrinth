use std::sync::mpsc;
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
                            position::Position::new(player_position.0, player_position.1), &healths, &covers) {
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
        let (mut map, entities, dead, mut drawables, mut grabbables, players, mut positions) = arg.fetch(|world| {
            (
                world.write_resource::<map::Map>(),
                world.entities(),
                world.read::<ai::Dead>(),
                world.write::<drawable::StaticDrawable>(),
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
                // TODO: randomize drop chance, use marker component
                // to control if an entity even drops stuff
                let corpse = arg.create();
                drawables.insert(corpse, drawable::StaticDrawable {
                    tc: '␣'.into(),
                });
                to_create.push((corpse, *position));
                // grabbables.insert(corpse, player::Grabbable())
                map.fill(corpse, position.x, position.y);
            }
        }

        for (entity, position) in to_create {
            positions.insert(entity, position);
        }
    }
}
