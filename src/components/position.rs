use specs;
use voodoo;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(map: &mut super::map::Map, x: usize, y: usize) -> Option<Position> {
        if map.passable(x, y) {
            map.fill(x, y);
            Some(Position {
                x: x,
                y: y,
            })
        }
        else {
            None
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize, map: &mut super::map::Map) -> Result<(), ()> {
        if map.passable(x, y) {
            map.vacate(self.x, self.y);
            self.x = x;
            self.y = y;
            map.fill(x, y);
            Ok(())
        }
        else {
            Err(())
        }
    }

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
