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
