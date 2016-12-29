use specs;

#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct Attack {
    pub damage: (usize, usize),
    pub accuracy: usize,
}

impl specs::Component for Attack {
    type Storage = specs::VecStorage<Attack>;
}
