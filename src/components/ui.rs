use specs::{self, Join};
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

pub struct InfoPanelResource {
    pub window: Window,
}

impl InfoPanelSystem {
    pub fn new() -> InfoPanelSystem {
        InfoPanelSystem {
        }
    }
}

impl InfoPanelResource {
    pub fn new(window: Window) -> InfoPanelResource {
        InfoPanelResource {
            window: window,
        }
    }
}

impl specs::System<()> for InfoPanelSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        let (mut res, focus, health) = arg.fetch(|world| {
            (world.write_resource::<InfoPanelResource>(), world.read::<Focus>(), world.read::<super::health::Health>())
        });

        res.window.print_at(Point::new(1, 0), "HP: ");
        let mut s: FormattedString = "    10/10     ".into();
        s.bg = Some(ColorValue::Red);
        res.window.print_at(Point::new(5, 0), s);
    }
}
