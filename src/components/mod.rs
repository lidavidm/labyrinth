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

use specs::World;

pub mod ai;
pub mod camera;
pub mod combat;
pub mod drawable;
pub mod health;
pub mod input;
pub mod map;
pub mod player;
pub mod position;
pub mod ui;

pub fn register_all(world: &mut World) {
    world.register::<ai::ChaseBehavior>();
    world.register::<ai::Dead>();

    world.register::<camera::Camera>();

    world.register::<combat::Attack>();
    world.register::<combat::DamageReduction>();

    world.register::<drawable::LineDrawable>();
    world.register::<drawable::StaticDrawable>();
    world.register::<drawable::DrawableRender>();

    world.register::<health::Cover>();
    world.register::<health::Health>();

    world.register::<input::Movable>();

    world.register::<player::DropsLoot>();
    world.register::<player::Equip>();
    world.register::<player::Grabbable>();
    world.register::<player::Inventory>();
    world.register::<player::Player>();

    world.register::<map::MapRender>();
    world.register::<map::MapBuilder>();

    world.register::<position::Position>();

    world.register::<ui::Focus>();
}
