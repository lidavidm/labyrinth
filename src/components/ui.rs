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

        for (_, health) in (&focus, &health).iter() {
            res.window.print_at(Point::new(1, 0), "Health ");
            res.window.print_at(Point::new(1, 1), "Stamina");
            let h = format!("{:30}", format!("{}/{}", health.health, health.max_health));
            let s = format!("{:30}", format!("{}/{}", health.stamina, health.max_stamina));
            let mut hfs: FormattedString = (&h).into();
            let mut sfs: FormattedString = (&s).into();
            hfs.bg = Some(ColorValue::Red);
            sfs.bg = Some(ColorValue::Green);
            res.window.print_at(Point::new(9, 0), hfs);
            res.window.print_at(Point::new(9, 1), sfs);
        }
    }
}
