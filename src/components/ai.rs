use specs;

/// This entity chases the player if spotted.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct ChaseBehavior {
    pub spotted: Option<(usize, usize)>,
}

/// This entity is dead and should not be processed by AI.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct Dead;

impl ChaseBehavior {
    pub fn new() -> ChaseBehavior {
        ChaseBehavior {
            spotted: None,
        }
    }
}

impl specs::Component for ChaseBehavior {
    type Storage = specs::VecStorage<ChaseBehavior>;
}

impl specs::Component for Dead {
    type Storage = specs::NullStorage<Dead>;
}
