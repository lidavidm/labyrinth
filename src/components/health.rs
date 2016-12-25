use specs;

pub struct Health {
    pub health: usize,
    pub stamina: usize,
    pub max_health: usize,
    pub max_stamina: usize,
}

impl Health {
    pub fn new(health: usize, max_health: usize, stamina: usize, max_stamina: usize) -> Health {
        Health {
            health: health,
            stamina: stamina,
            max_health: max_health,
            max_stamina: max_stamina,
        }
    }
}

impl specs::Component for Health {
    type Storage = specs::VecStorage<Health>;
}
