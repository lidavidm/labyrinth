use std::sync::mpsc;

use rand::{self, Rng};
use specs::{self, Join};

use ::components::{ai, combat, health};

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
        let (entities, mut chasers, mut attacked, mut healths) = arg.fetch(|world| {
            (
                world.entities(),
                world.write::<ai::ChaseBehavior>(),
                world.write::<combat::Attack>(),
                world.write::<health::Health>(),
            )
        });

        let mut to_delete = vec![];
        for (entity, attack, health) in (&entities, &mut attacked, &mut healths).iter() {
            let damage = rand::thread_rng().gen_range(attack.damage.0, attack.damage.1);
            if damage >= health.health {
                self.message_queue.send(format!("Hit for {} damage, killed!", damage)).unwrap();
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
