use specs;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Attack {
    pub damage: (usize, usize),
    pub accuracy: usize,
    pub source: specs::Entity,
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct DamageReduction {
    pub value: usize,
}

impl specs::Component for Attack {
    type Storage = specs::VecStorage<Attack>;
}

impl specs::Component for DamageReduction {
    type Storage = specs::VecStorage<DamageReduction>;
}
