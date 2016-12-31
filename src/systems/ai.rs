use std::sync::mpsc;
use specs::{self, Join};

use ::components::{ai, combat, health, map, player, position};
use ::components::input::OffsetMovable;
use ::util;

pub struct AiSystem {
    ai_begin: mpsc::Receiver<()>,
    ai_end: mpsc::Sender<()>,
}

pub struct DeadSystem {
}

impl AiSystem {
    pub fn new(ai_begin: mpsc::Receiver<()>, ai_end: mpsc::Sender<()>) -> AiSystem {
        AiSystem {
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

        let (mut map, entities, mut chase_behaviors, mut attacked, dead, healths, equipped, players, mut positions) = arg.fetch(|world| {
            (
                world.write_resource::<map::Map>(),
                world.entities(),
                world.write::<ai::ChaseBehavior>(),
                world.write::<combat::Attack>(),
                world.read::<ai::Dead>(),
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
                        match ::util::combat::resolve(&map, me, &equip, *position, position::Position::new(player_position.0, player_position.1), |entity| {
                            healths.get(entity).is_some()
                        }) {
                            ::util::combat::CombatResult::NothingEquipped => {
                                // self.message_queue.send("You have nothing equipped!".into()).unwrap();
                            }
                            ::util::combat::CombatResult::Miss => {
                                // self.message_queue.send("You missed!".into()).unwrap();
                            }
                            ::util::combat::CombatResult::HitNothing => {
                                // self.message_queue.send("You hit nothing.".into()).unwrap();
                            }
                            ::util::combat::CombatResult::HitEnvironment => {
                                // self.message_queue.send("You hit a wall.".into()).unwrap();
                            }
                            ::util::combat::CombatResult::HitEntity(target, pos, attack) => {
                                // self.message_queue.send(format!("Targeted {}, {}", pos.x, pos.y)).unwrap();
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
    pub fn new() -> DeadSystem {
        DeadSystem {}
    }
}

impl specs::System<()> for DeadSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut map, entities, dead, positions) = arg.fetch(|world| {
            (
                world.write_resource::<map::Map>(),
                world.entities(),
                world.read::<ai::Dead>(),
                world.read::<position::Position>(),
            )
        });

        for (entity, position, _) in (&entities, &positions, &dead).iter() {
            arg.delete(entity);
            map.vacate(position.x, position.y);
            // TODO: drop a corpse or something
        }
    }
}
