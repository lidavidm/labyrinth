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
        let (entities, mut map, players, mut chasers, mut attacked, mut healths, mut positions) = arg.fetch(|world| {
            (
                world.entities(),
                world.write_resource::<map::Map>(),
                world.read::<player::Player>(),
                world.write::<ai::ChaseBehavior>(),
                world.write::<combat::Attack>(),
                world.write::<health::Health>(),
                world.write::<position::Position>(),
            )
        });

        let mut to_delete = vec![];
        let mut to_kill = vec![];
        for (entity, attack, health, position) in (&entities, &mut attacked, &mut healths, &positions).iter() {
            let damage = rand::thread_rng().gen_range(attack.damage.0, attack.damage.1);
            if damage >= health.health {
                self.message_queue.send(format!("Hit for {} damage, killed!", damage)).unwrap();
                map.vacate(position.x, position.y);
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
            // Remove the position component immediately, then let
            // specs handle fully deleting it later. Otherwise systems
            // will still try and process the entity.
            // TODO: some sort of Alive or Dead marker component?
            positions.remove(entity);
            arg.delete(entity);
        }
    }
}
