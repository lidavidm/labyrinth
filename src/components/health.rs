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
