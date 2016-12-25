use specs;

pub struct Health {
    pub health: usize,
    pub stamina: usize,
    pub max_health: usize,
    pub max_stamina: usize,
}

impl specs::Component for Health {
    type Storage = specs::VecStorage<Health>;
}
