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

extern crate env_logger;
extern crate log;
extern crate log_panics;
extern crate rand;
extern crate specs;
extern crate termion;
extern crate time;
extern crate voodoo;

pub mod components;
pub mod screen;
pub mod systems;
pub mod ui;
pub mod util;

use std::sync::mpsc;
use std::thread;

const MS: u64 = 1_000_000;
const TICK_TIME: u64 = MS * 25;

const WIDTH: u16 = 80;
const HEIGHT: u16 = 24;

const MAP_WIDTH: u16 = 38;
const MAP_HEIGHT: u16 = 18;

fn async_events<R: std::io::Read + Send + 'static>(stdin: R) -> mpsc::Receiver<std::io::Result<termion::event::Event>> {
    use termion::input::TermRead;

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for event in stdin.events() {
            tx.send(event).expect("Couldn't transmit event!");
        }
    });

    rx
}

fn run() -> f64 {
    use std::time::Duration;

    use voodoo::color::ColorValue;
    use voodoo::terminal::{Mode, Terminal};

    env_logger::init().unwrap();
    log_panics::init();

    let mut world = specs::World::new();
    components::register_all(&mut world);
    let mut planner = specs::Planner::<()>::new(world, 2);

    // Initialize the console
    let (terminal, stdin, mut stdout) = Terminal::new();
    terminal.cursor(Mode::Disabled);
    terminal.clear_color(ColorValue::Black);

    let mut compositor = voodoo::compositor::Compositor::new(WIDTH, HEIGHT);

    let rx = async_events(stdin);

    let mut last_tick = time::precise_time_ns();
    let mut dt = 0;
    let mut avg_frame_time = 0.0;
    let mut frames: u64 = 0;

    let mut state = screen::StateManager::new(&mut planner, screen::StateTransition::Game);

    'main: loop {
        for event in rx.try_iter() {
            if let Ok(event) = event {
                state.dispatch(event);
            }
        }

        let now = time::precise_time_ns();
        let old_tick = last_tick;
        dt += now - last_tick;
        last_tick = now;

        while dt > TICK_TIME {
            dt -= TICK_TIME;
            planner.dispatch(());
        }

        state.render(&mut planner, &mut compositor);
        compositor.display(&mut stdout);
        if state.update(&mut planner) {
            break 'main;
        }
        thread::sleep(Duration::from_millis((TICK_TIME - dt) / MS));

        let frame_time = time::precise_time_ns() - old_tick;
        avg_frame_time = ((frames as f64 * avg_frame_time) + frame_time as f64) / (frames as f64 + 1.0);
        frames += 1;
    }

    avg_frame_time
}

fn main() {
    let avg_frame_time = run();
    println!("Average frame time: {:03.03} ms", avg_frame_time / MS as f64);
}
