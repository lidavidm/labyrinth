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

use ::components::{ai, combat, health, player, position};

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
        let (entities, players, mut chasers, mut dead, mut attacked, dr, mut healths, positions) = arg.fetch(|world| {
            (
                world.entities(),
                world.read::<player::Player>(),
                world.write::<ai::ChaseBehavior>(),
                world.write::<ai::Dead>(),
                world.write::<combat::Attack>(),
                world.read::<combat::DamageReduction>(),
                world.write::<health::Health>(),
                world.read::<position::Position>(),
            )
        });

        let mut to_delete = vec![];
        let mut to_kill = vec![];
        for (entity, attack, health) in (&entities, &mut attacked, &mut healths).iter() {
            let mut damage = rand::thread_rng().gen_range(attack.damage.0, attack.damage.1);

            if let Some(dr) = dr.get(entity) {
                let orig = damage;
                damage = damage.saturating_sub(dr.value);
                if dr.value > 0 {
                    self.message_queue.send(format!("DR: {} -> {}", orig, damage)).unwrap();
                }
            }

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
