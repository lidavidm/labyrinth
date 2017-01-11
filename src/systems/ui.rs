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

use std::sync::mpsc;

use specs::{self, Join};
use voodoo::color::ColorValue;
use voodoo::window::{FormattedString, Point, Window};

use ::components;

macro_rules! make_resource {
    ($name: ident) => {
        pub struct $name {
            pub window: Window,
        }

        impl $name {
            pub fn new(window: Window) -> $name {
                $name {
                    window: window
                }
            }
        }
    }
}

pub struct InfoPanelSystem {
}

make_resource!(InfoPanelResource);
make_resource!(CommandPanelResource);
make_resource!(MessagesPanelResource);
make_resource!(InventoryPanelResource);

impl InfoPanelSystem {
    pub fn new() -> InfoPanelSystem {
        InfoPanelSystem {
        }
    }
}

impl specs::System<()> for InfoPanelSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut res, focus, health) = arg.fetch(|world| {
            (world.write_resource::<InfoPanelResource>(), world.read::<components::ui::Focus>(), world.read::<components::health::Health>())
        });

        for (_, health) in (&focus, &health).iter() {
            res.window.print_at(Point::new(1, 0), "Health ");
            let h = format!("{:30}", format!("{}/{}", health.health, health.max_health));
            let mut hfs: FormattedString = (&h).into();
            hfs.bg = Some(ColorValue::Red);
            res.window.print_at(Point::new(9, 0), hfs);
        }
    }
}

pub struct MessagesPanelSystem {
    pub messages: Vec<String>,
    pub incoming: mpsc::Receiver<String>,
    pub cursor: Point,
}

impl MessagesPanelSystem {
    pub fn new() -> (MessagesPanelSystem, mpsc::Sender<String>) {
        let (tx, rx) = mpsc::channel();
        (MessagesPanelSystem {
            messages: Vec::new(),
            incoming: rx,
            cursor: Point::new(0, 0),
        }, tx)
    }
}

impl specs::System<()> for MessagesPanelSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let mut res = arg.fetch(|world| {
            world.write_resource::<MessagesPanelResource>()
        });

        let mut need_repaint = false;
        for message in self.incoming.try_iter() {
            if self.cursor.y < res.window.height - 1 {
                res.window.print_at(self.cursor, &message);
                self.cursor.x = 0;
                self.cursor.y += 1;
            }
            else {
                need_repaint = true;
            }
            self.messages.push(message);
        }

        if need_repaint {
            res.window.clear();
            let base = self.messages.len() - res.window.height as usize;
            for y in 0..res.window.height {
                res.window.print_at(Point::new(0, y), &self.messages[base + y as usize]);
            }
        }
    }
}
