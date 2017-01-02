use rand::{self, Rng};
use specs::{self, Entity};

use ::components::combat::Attack;
use ::components::health::Health;
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

type S<C> = specs::Storage<C, ::std::ops::Deref<Target=specs::Allocator>, ::std::ops::Deref<Target=specs::MaskedStorage<C>>>;

pub fn resolve<A, D>(map: &Map, attacker: Entity, equip: &Equip,
                  origin: Position, target: Position, targetable: &specs::Storage<Health, A, D>) -> CombatResult
    where A: ::std::ops::Deref<Target=specs::Allocator>,
          D: ::std::ops::Deref<Target=specs::MaskedStorage<Health>>, {
    let points = ::util::bresenham(origin, target);

    let attack = if let Some(Item {
        kind: ItemKind::Weapon {
            damage, accuracy
        },
        ..
    }) = equip.left_hand {
        // TODO: change this to calculate accuracy as we walk the
        // line, so we can account for cover
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
            if *target == origin {
                continue;
            }
            if let Some(entity) = map.contents(target.x, target.y) {
                if let Some(_) = targetable.get(entity) {
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
