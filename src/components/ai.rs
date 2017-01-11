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

/// This entity chases the player if spotted.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct ChaseBehavior {
    pub spotted: Option<(usize, usize)>,
}

/// This entity is dead and should not be processed by AI.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct Dead;

impl ChaseBehavior {
    pub fn new() -> ChaseBehavior {
        ChaseBehavior {
            spotted: None,
        }
    }
}

impl specs::Component for ChaseBehavior {
    type Storage = specs::VecStorage<ChaseBehavior>;
}

impl specs::Component for Dead {
    type Storage = specs::NullStorage<Dead>;
}
