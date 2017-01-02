use rand::{self, Rng};
use specs::{self, Entity};

use ::components::combat::Attack;
use ::components::health::{Cover, Health};
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

pub trait HasStorage<C> {
    fn check(&self, entity: Entity) -> bool;
    fn get(&self, entity: Entity) -> Option<&C>;
}

impl<C, A, D> HasStorage<C> for specs::Storage<C, A, D>
    where
    C: specs::Component,
    A: ::std::ops::Deref<Target=specs::Allocator>,
    D: ::std::ops::Deref<Target=specs::MaskedStorage<C>>
{
    fn check(&self, entity: Entity) -> bool {
        if let Some(_) = self.get(entity) {
            true
        }
        else {
            false
        }
    }

    fn get(&self, entity: Entity) -> Option<&C> {
        specs::Storage::get(self, entity)
    }
}

pub fn resolve<H, C>(map: &Map, attacker: Entity, equip: &Equip,
                     origin: Position, target: Position,
                     targetable: &H,
                     cover: &C) -> CombatResult
    where H: HasStorage<Health>, C: HasStorage<Cover> {
    let points = ::util::bresenham(origin, target);

    let attack = if let Some(Item {
        kind: ItemKind::Weapon {
            damage, accuracy
        },
        ..
    }) = equip.primary {
        Some(Attack {
            damage: damage,
            accuracy: accuracy,
            source: attacker,
        })
    }
    else {
        None
    };

    if let Some(attack) = attack {
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

                if targetable.check(entity) && rand::thread_rng().gen_range(0, 1000) < attack.accuracy as i32 + accuracy_penalty {
                    return CombatResult::HitEntity(entity, *target, attack);
                }
                else if index == last {
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
