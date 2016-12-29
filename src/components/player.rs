use specs;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum ItemKind {
    Weapon {
        damage: (usize, usize),
        accuracy: usize,
    },
}

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
}

#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct Equip {
    pub left_hand: Option<Item>,
}

/// This entity is the player.
#[derive(Clone,Debug,Default,Eq,PartialEq)]
pub struct Player {
}

impl Player {
    pub fn new() -> Player {
        Player {
        }
    }
}

impl Equip {
    pub fn new() -> Equip {
        Equip {
            left_hand: None,
        }
    }
}

impl specs::Component for Player {
    type Storage = specs::VecStorage<Player>;
}

impl specs::Component for Equip {
    type Storage = specs::VecStorage<Equip>;
}
