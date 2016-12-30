use rand::{self, Rng};
use specs::Entity;

use ::components::combat::Attack;
use ::components::map::Map;
use ::components::player::{Equip, Item, ItemKind};
use ::components::position::Position;

pub enum CombatResult {
    NothingEquipped,
    Miss,
    HitNothing,
    HitEnvironment,
    HitEntity(Entity, Position, Attack),
}

pub fn resolve<F>(map: &Map, attacker: Entity, equip: &Equip,
                  origin: Position, target: Position, targetable: F) -> CombatResult
    where F: Fn(Entity) -> bool {
    let points = ::util::bresenham(origin, target);

    let attack = if let Some(Item {
        kind: ItemKind::Weapon {
            damage, accuracy
        },
        ..
    }) = equip.left_hand {
        if rand::thread_rng().gen_range(0, 1000) < accuracy {
            Some(Attack {
                damage: damage,
                accuracy: accuracy,
                source: attacker,
            })
        }
        else {
            return CombatResult::Miss;
        }
    }
    else {
        None
    };

    if let Some(attack) = attack {
        for target in points.iter().skip(1) {
            if let Some(entity) = map.contents(target.x, target.y) {
                if targetable(entity) {
                    return CombatResult::HitEntity(entity, *target, attack);
                }
            }
            if !map.passable(target.x, target.y) {
                return CombatResult::HitEnvironment;
            }
        }

        return CombatResult::HitNothing;
    }
    else {
        CombatResult::NothingEquipped
    }
}
