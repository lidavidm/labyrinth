use specs;

pub struct Health {
    pub health: usize,
    pub stamina: f32,
    pub max_health: usize,
    pub max_stamina: f32,
}

impl Health {
    pub fn new(health: usize, max_health: usize, stamina: f32, max_stamina: f32) -> Health {
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
