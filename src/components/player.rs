use specs;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum ItemKind {
    Weapon {
        damage: (usize, usize),
        accuracy: usize,
        range: usize,
    },
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Grabbable(Item);

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

impl Equip {
    pub fn new() -> Equip {
        Default::default()
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

impl specs::Component for Player {
    type Storage = specs::VecStorage<Player>;
}

impl specs::Component for Equip {
    type Storage = specs::VecStorage<Equip>;
}

impl specs::Component for Inventory {
    type Storage = specs::VecStorage<Inventory>;
}
