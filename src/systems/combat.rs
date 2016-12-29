use std::sync::mpsc;

use rand::{self, Rng};
use specs::{self, Join};

use ::components::{ai, combat, health, map, position};

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
        let (entities, mut map, mut chasers, mut attacked, mut healths, positions) = arg.fetch(|world| {
            (
                world.entities(),
                world.write_resource::<map::Map>(),
                world.write::<ai::ChaseBehavior>(),
                world.write::<combat::Attack>(),
                world.write::<health::Health>(),
                world.read::<position::Position>(),
            )
        });

        let mut to_delete = vec![];
        for (entity, attack, health, position) in (&entities, &mut attacked, &mut healths, &positions).iter() {
            let damage = rand::thread_rng().gen_range(attack.damage.0, attack.damage.1);
            if damage >= health.health {
                self.message_queue.send(format!("Hit for {} damage, killed!", damage)).unwrap();
                map.vacate(position.x, position.y);
                arg.delete(entity);
            }
            else {
                // TODO: if not player, add chase behavior
                health.health -= damage;
                self.message_queue.send(format!("Hit for {} damage, {} left", damage, health.health)).unwrap();
            }
            to_delete.push(entity);
        }

        for entity in to_delete {
            attacked.remove(entity);
        }
    }
}
