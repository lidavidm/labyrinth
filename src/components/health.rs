use specs;

pub struct Health {
    pub health: usize,
    pub base_health: usize,
    pub max_health: usize,
}

pub struct Cover {
    pub penalty: i32,
}

impl Health {
    pub fn new(health: usize, base_health: usize) -> Health {
        Health {
            health: health,
            base_health: base_health,
            max_health: base_health,
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
