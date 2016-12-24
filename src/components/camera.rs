use specs;
use voodoo::window::Point;

pub struct Camera {
    // TODO: Point(u16) isn't adequate for maps
    pub position: Point,
    pub view: (u16, u16),
    pub area: (usize, usize),
}

impl Camera {
    pub fn new(view: (u16, u16), area: (usize, usize)) -> Camera {
        Camera {
            position: Point::new(0, 0),
            view: view,
            area: area,
        }
    }

    pub fn max_x(&self) -> u16 {
        self.area.0 as u16 - self.view.0
    }

    pub fn max_y(&self) -> u16 {
        self.area.1 as u16 - self.view.1
    }

    pub fn is_visible(&self, position: &super::position::Position) -> bool {
        return position.x >= self.position.x as usize &&
            position.y >= self.position.y as usize &&
            position.x < (self.position.x + self.view.0) as usize &&
            position.y < (self.position.y + self.view.1) as usize;
    }
}

impl specs::Component for Camera {
    type Storage = specs::VecStorage<Camera>;
}
