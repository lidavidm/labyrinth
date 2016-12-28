use specs;

/// This entity chases the player if spotted.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct ChaseBehavior {
    pub spotted: Option<(usize, usize)>,
}

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
