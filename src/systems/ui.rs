use std::sync::mpsc;

use specs::{self, Join};
use voodoo;
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
            res.window.print_at(Point::new(1, 1), "Stamina");
            let h = format!("{:30}", format!("{}/{}", health.health, health.max_health));
            let s = format!("{:30}", format!("{}/{}", health.stamina as usize, health.max_stamina as usize));
            let mut hfs: FormattedString = (&h).into();
            let mut sfs: FormattedString = (&s).into();
            hfs.bg = Some(ColorValue::Red);
            sfs.bg = Some(ColorValue::Green);
            res.window.print_at(Point::new(9, 0), hfs);
            res.window.print_at(Point::new(9, 1), sfs);
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
