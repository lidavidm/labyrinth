use specs;

pub struct Health {
    pub health: usize,
    pub stamina: f32,
    pub max_health: usize,
    pub max_stamina: f32,
}

pub struct Cover {
    pub penalty: i32,
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

impl Cover {
    pub fn new(penalty: i32) -> Cover {
        Cover {
            penalty: penalty,
        }
    }
}

impl specs::Component for Health {
    type Storage = specs::VecStorage<Health>;
}

impl specs::Component for Cover {
    type Storage = specs::VecStorage<Cover>;
}
