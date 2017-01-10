use rand::{self, Rng};
use specs::Entity;

use super::HasStorage;

use ::components::combat::Attack;
use ::components::health::{Cover, Health};
use ::components::map::Map;
use ::components::player::{Equip, Item, ItemKind};
use ::components::position::Position;

pub enum CombatResult {
    /// No weapon
    NothingEquipped,
    /// You missed
    Miss,
    /// You targeted nothing
    HitNothing,
    /// You hit a wall
    HitEnvironment,
    /// You hit
    HitEntity(Entity, Position, Attack),
    /// You're using a melee weapon
    OutOfRange,
}

pub fn resolve<H, C>(map: &Map, attacker: Entity, equip: &Equip,
                     origin: Position, target: Position,
                     is_melee: bool,
                     targetable: &H,
                     cover: &C) -> CombatResult
    where H: HasStorage<Health>, C: HasStorage<Cover> {
    let points = ::util::bresenham(origin, target);

    let attack = if let &Some(Item {
        kind: ItemKind::Weapon {
            damage, accuracy, range,
        },
        ..
    }) = if is_melee { &equip.secondary } else { &equip.primary } {
        Some((Attack {
            damage: damage,
            accuracy: accuracy,
            source: attacker,
        }, range))
    }
    else {
        None
    };

    if let Some((attack, range)) = attack {
        let mut accuracy_penalty = 0;

        let last = points.len() - 1;
        for (index, target) in points.iter().enumerate() {
            if *target == origin {
                continue;
            }
            if let Some(entity) = map.contents(target.x, target.y) {
                // If there is cover, and they are not targeting it,
                // influence the accuracy
                if index != last {
                    if let Some(&Cover { penalty }) = cover.get(entity) {
                        accuracy_penalty += penalty;
                        // Don't continue; here - you have a chance to hit
                        // the cover
                    }
                }

                let dist = ::util::distance2((origin.x, origin.y), (target.x, target.y));
                if range == 0 && dist > 1 {
                    return CombatResult::OutOfRange;
                }

                if dist > range * range {
                    accuracy_penalty -= 100;
                }

                if targetable.check(entity) && rand::thread_rng().gen_range(0, 1000) < attack.accuracy as i32 + accuracy_penalty {
                    return CombatResult::HitEntity(entity, *target, attack);
                }
                else if index == last || range == 0 && dist <= 1 {
                    return CombatResult::Miss;
                }
            }
            if !map.occupable(target.x, target.y) {
                return CombatResult::HitEnvironment;
            }
        }

        return CombatResult::HitNothing;
    }
    else {
        CombatResult::NothingEquipped
    }
}
