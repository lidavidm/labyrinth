use specs;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Attack {
    pub damage: (usize, usize),
    pub accuracy: usize,
    pub source: specs::Entity,
}

impl specs::Component for Attack {
    type Storage = specs::VecStorage<Attack>;
}
