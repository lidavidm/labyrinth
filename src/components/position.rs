// Copyright (C) 2016-2017 David Li

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use specs;
use voodoo;

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position {
            x: x,
            y: y,
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize, map: &mut super::map::Map) -> Result<(), ()> {
        if map.passable(x, y) {
            let entity = map.vacate(self.x, self.y).unwrap();
            self.x = x;
            self.y = y;
            map.fill(entity, x, y);
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

impl super::input::OffsetMovable for Position {
    fn move_by(&mut self, offset: (i32, i32), map: &mut super::map::Map) -> Result<(), ()> {
        let new_x = self.x as i32 + offset.0;
        let new_y = self.y as i32 + offset.1;

        let new_x = if new_x < 0 { 0 } else { new_x as usize };
        let new_y = if new_y < 0 { 0 } else { new_y as usize };

        self.move_to(new_x, new_y, map)
    }
}
