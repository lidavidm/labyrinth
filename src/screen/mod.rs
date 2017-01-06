use std::sync::mpsc;

use specs;
use termion;
use voodoo::compositor::Compositor;

mod game;
mod game_over;
pub use self::game::GameScreen;
pub use self::game_over::GameOverScreen;

pub type TransitionChannel = mpsc::Sender<StateTransition>;

pub trait Screen {
    fn setup(planner: &mut specs::Planner<()>, transitions: mpsc::Sender<StateTransition>) -> Self;

    fn dispatch(&mut self, event: termion::event::Event);

    fn render(&mut self, planner: &mut specs::Planner<()>, compositor: &mut Compositor);

    fn teardown(&mut self, planner: &mut specs::Planner<()>) {
        planner.systems.clear();
    }
}

pub enum State {
    Game(GameScreen),
    GameOver(GameOverScreen),
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum StateTransition {
    Game,
    GameOver,
    Quit,
}

impl StateTransition {
    pub fn make(&self, planner: &mut specs::Planner<()>, transitions: mpsc::Sender<StateTransition>) -> State {
        use self::StateTransition::*;

        match *self {
            Game => State::Game(GameScreen::setup(planner, transitions)),
            GameOver => State::GameOver(GameOverScreen::setup(planner, transitions)),
            Quit => panic!("Shouldn't construct this state"),
        }
    }
}

pub struct StateManager {
    state: State,
    transitions: (mpsc::Sender<StateTransition>, mpsc::Receiver<StateTransition>),
}

impl StateManager {
    pub fn new(planner: &mut specs::Planner<()>, default_state: StateTransition) -> StateManager {
        let transitions = mpsc::channel();
        StateManager {
            state: default_state.make(planner, transitions.0.clone()),
            transitions: transitions,
        }
    }

    pub fn update(&mut self, planner: &mut specs::Planner<()>) -> bool {
        if let Some(transition) = self.transitions.1.try_iter().last() {
            self.teardown(planner);
            if transition == StateTransition::Quit {
                return true;
            }
            self.state = transition.make(planner, self.transitions.0.clone());
        }
        false
    }

    pub fn dispatch(&mut self, event: termion::event::Event) {
        use self::State::*;

        match self.state {
            Game(ref mut screen) => screen.dispatch(event),
            GameOver(ref mut screen) => screen.dispatch(event),
        }
    }

    pub fn render(&mut self, planner: &mut specs::Planner<()>, compositor: &mut Compositor) {
        use self::State::*;

        match self.state {
            Game(ref mut screen) => screen.render(planner, compositor),
            GameOver(ref mut screen) => screen.render(planner, compositor),
        }
    }

    pub fn teardown(&mut self, planner: &mut specs::Planner<()>) {
        use self::State::*;

        match self.state {
            Game(ref mut screen) => screen.teardown(planner),
            GameOver(ref mut screen) => screen.teardown(planner),
        }
    }
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum SubScreenEvent<T> {
    Push(T),
    Pop,
}

impl<T: Copy> SubScreenEvent<T> {
    pub fn apply(&self, stack: &mut Vec<T>) {
        match self {
            &SubScreenEvent::Pop => {
                stack.pop();
            },
            &SubScreenEvent::Push(t) => stack.push(t),
        };
    }


    pub fn apply_all(channel: &mpsc::Receiver<SubScreenEvent<T>>, stack: &mut Vec<T>) {
        for item in channel.try_iter() {
            item.apply(stack);
        }
    }
}
