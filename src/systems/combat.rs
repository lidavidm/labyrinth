use std::sync::mpsc;

use rand::{self, Rng};
use specs::{self, Join};

use ::components::{ai, combat, health, map, player, position};

pub struct CombatSystem {
    message_queue: mpsc::Sender<String>,
}

impl CombatSystem {
    pub fn new(message_queue: mpsc::Sender<String>) -> CombatSystem {
        CombatSystem {
            message_queue: message_queue,
        }
    }
}

impl specs::System<()> for CombatSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (entities, players, mut chasers, mut dead, mut attacked, mut healths, positions) = arg.fetch(|world| {
            (
                world.entities(),
                world.read::<player::Player>(),
                world.write::<ai::ChaseBehavior>(),
                world.write::<ai::Dead>(),
                world.write::<combat::Attack>(),
                world.write::<health::Health>(),
                world.read::<position::Position>(),
            )
        });

        let mut to_delete = vec![];
        let mut to_kill = vec![];
        for (entity, attack, health) in (&entities, &mut attacked, &mut healths).iter() {
            let damage = rand::thread_rng().gen_range(attack.damage.0, attack.damage.1);
            if damage >= health.health {
                self.message_queue.send(format!("Hit for {} damage, killed!", damage)).unwrap();
                to_kill.push(entity);
            }
            else {
                // If not player, add chase behavior
                if let None = players.get(entity) {
                    if let Some(pos) = positions.get(attack.source) {
                        chasers.insert(entity, ai::ChaseBehavior {
                            spotted: Some((pos.x, pos.y)),
                        });
                    }
                }
                health.health -= damage;
                self.message_queue.send(format!("Hit for {} damage, {} left", damage, health.health)).unwrap();
            }
            to_delete.push(entity);
        }

        for entity in to_delete {
            attacked.remove(entity);
        }

        for entity in to_kill {
            dead.insert(entity, ai::Dead);
        }
    }
}
