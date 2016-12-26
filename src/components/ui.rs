use std::sync::mpsc;

use specs::{self, Join};
use termion::event::Key;
use voodoo::color::ColorValue;
use voodoo::window::{FormattedString, Point, Window};

/// This entity has the UI focus.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct Focus;

impl specs::Component for Focus {
    type Storage = specs::NullStorage<Focus>;
}

pub struct InfoPanelSystem {
}

pub struct CommandPanelSystem {
    pub inputs: mpsc::Receiver<Key>,
}

pub struct InfoPanelResource {
    pub window: Window,
}

pub struct CommandPanelResource {
    pub window: Window,
}

impl InfoPanelSystem {
    pub fn new() -> InfoPanelSystem {
        InfoPanelSystem {
        }
    }
}

impl CommandPanelSystem {
    pub fn new() -> (CommandPanelSystem, mpsc::Sender<Key>) {
        let (tx, rx) = mpsc::channel();
        (CommandPanelSystem {
            inputs: rx,
        }, tx)
    }
}

impl InfoPanelResource {
    pub fn new(window: Window) -> InfoPanelResource {
        InfoPanelResource {
            window: window,
        }
    }
}

impl CommandPanelResource {
    pub fn new(window: Window) -> CommandPanelResource {
        CommandPanelResource {
            window: window,
        }
    }
}

impl specs::System<()> for InfoPanelSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut res, focus, health) = arg.fetch(|world| {
            (world.write_resource::<InfoPanelResource>(), world.read::<Focus>(), world.read::<super::health::Health>())
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

impl specs::System<()> for CommandPanelSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut res, focus, health) = arg.fetch(|world| {
            (world.write_resource::<CommandPanelResource>(), world.read::<Focus>(), world.read::<super::health::Health>())
        });

        res.window.print_at(Point::new(0, 0), "WASD—Move");
        res.window.print_at(Point::new(0, 1), "   1—Examine");
        res.window.print_at(Point::new(0, 2), "   2—Interact");
        res.window.print_at(Point::new(0, 3), "   3—Fire");

        res.window.print_at(Point::new(14, 0), "I—Inventory");
        res.window.print_at(Point::new(14, 1), "B—Build");
        res.window.print_at(Point::new(14, 2), "T—Rest");
        res.window.print_at(Point::new(14, 3), "F—Journal");
    }
}
