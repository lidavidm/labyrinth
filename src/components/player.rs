use specs;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum ItemKind {
    Weapon {
        damage: (usize, usize),
        accuracy: usize,
        range: usize,
    },
    Armor {
        health: usize,
        damage_reduction: usize,
    },
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub enum ItemSlot {
    Primary,
    Secondary,
    Head,
    Body,
    Legs,
    Feet,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
    pub slot: Option<ItemSlot>,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Grabbable(pub Item);

#[derive(Debug,Eq,PartialEq)]
pub struct DropsLoot {
    pub chance: usize,
    pub items: Vec<Item>,
}

#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct Equip {
    pub primary: Option<Item>,
    pub secondary: Option<Item>,
    pub head: Option<Item>,
    pub body: Option<Item>,
    pub legs: Option<Item>,
    pub feet: Option<Item>,
}

pub struct Inventory {
    pub contents: Vec<Item>,
}

/// This entity is the player.
#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct Player {
}

impl ::ui::ListRenderable for Item {
    fn render(&self) -> Vec<String> {
        let mut result = vec![self.name.clone()];

        match self.kind {
            ItemKind::Weapon { damage, accuracy, range } => {
                result.push(format!("Damage: {} to {}", damage.0, damage.1 - 1));
                result.push(format!("Accuracy: {}/1000", accuracy));
                if range > 0 {
                    result.push(format!("Range: {}", range));
                }
                else {
                    result.push("Range: Melee".into());
                }
            }
            ItemKind::Armor { health, damage_reduction } => {
                if let Some(ref slot) = self.slot {
                    result.push(format!("{:?} Armor", slot));
                }
                result.push(format!("Health Bonus: {}", health));
                result.push(format!("Damage Reduction: {}", damage_reduction));
            }
        }

        result
    }
}

impl Player {
    pub fn new() -> Player {
        Player {
        }
    }
}

impl DropsLoot {
    pub fn new<I: Into<Vec<Item>>>(chance: usize, items: I) -> DropsLoot {
        DropsLoot {
            chance: chance,
            items: items.into(),
        }
    }
}

impl Equip {
    pub fn new() -> Equip {
        Default::default()
    }

    pub fn equip(&mut self, item: Item) -> Option<Item> {
        use self::ItemSlot::*;

        match item.slot {
            Some(Primary) => {
                let old = self.primary.clone();
                self.primary = Some(item);
                old
            }
            Some(Secondary) => {
                let old = self.secondary.clone();
                self.secondary = Some(item);
                old
            }
            Some(Head) => {
                let old = self.head.clone();
                self.head = Some(item);
                old
            }
            Some(Body) => {
                let old = self.body.clone();
                self.body = Some(item);
                old
            }
            Some(Legs) => {
                let old = self.legs.clone();
                self.legs = Some(item);
                old
            }
            Some(Feet) => {
                let old = self.feet.clone();
                self.feet = Some(item);
                old
            }
            None => None,
        }
    }
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            contents: Vec::new(),
        }
    }
}

impl specs::Component for Grabbable {
    type Storage = specs::VecStorage<Grabbable>;
}

impl specs::Component for DropsLoot {
    type Storage = specs::HashMapStorage<DropsLoot>;
}

impl specs::Component for Player {
    type Storage = specs::HashMapStorage<Player>;
}

impl specs::Component for Equip {
    type Storage = specs::VecStorage<Equip>;
}

impl specs::Component for Inventory {
    type Storage = specs::VecStorage<Inventory>;
}
