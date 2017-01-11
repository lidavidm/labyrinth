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
use termion;
use voodoo::compositor::Compositor;
use voodoo::window::{Point, Window};

use ::{WIDTH, HEIGHT};

pub struct GameOverScreen {
    window: Window,
    transitions: super::TransitionChannel,
}

impl ::screen::Screen for GameOverScreen {
    fn setup(planner: &mut specs::Planner<()>, transitions: super::TransitionChannel) -> GameOverScreen {
        GameOverScreen {
            window: Window::new(Point::new(0, 0), WIDTH, HEIGHT),
            transitions: transitions,
        }
    }

    fn dispatch(&mut self, _: termion::event::Event) {
        self.transitions.send(super::StateTransition::Quit).unwrap();
    }

    fn render(&mut self, planner: &mut specs::Planner<()>, compositor: &mut Compositor) {
        self.window.clear();
        self.window.border();
        self.window.print_at(Point::new(1, 1), "GAME OVER");
        self.window.print_at(Point::new(1, 3), "Press Esc to exit");
        self.window.refresh(compositor);
    }
}
