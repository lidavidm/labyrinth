use specs;
use voodoo;

pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn relative_to(&self, camera: &super::camera::Camera) -> Option<voodoo::window::Point> {
        if camera.is_visible(self) {
            Some(voodoo::window::Point::new(self.x as u16 - camera.position.x, self.y as u16 - camera.position.y))
        }
        else {
            None
        }
    }
}

impl specs::Component for Position {
    type Storage = specs::VecStorage<Position>;
}
