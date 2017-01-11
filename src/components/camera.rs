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

use std::cmp::min;

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

    pub fn center_on(&mut self, x: u16, y: u16) {
        let x = (x as i32) - (self.view.0 as i32) / 2;
        let y = (y as i32) - (self.view.1 as i32) / 2;

        self.position.x = if x < 0 { 0 } else { min(self.max_x(), x as u16) };
        self.position.y = if y < 0 { 0 } else { min(self.max_y(), y as u16) };
    }
}

impl specs::Component for Camera {
    type Storage = specs::VecStorage<Camera>;
}
