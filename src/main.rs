extern crate specs;
extern crate termion;
extern crate time;
extern crate voodoo;

mod components;

use std::sync::mpsc;
use std::thread;

const MS: u64 = 1_000_000;
const TICK_TIME: u64 = MS * 25;

fn run() -> f64 {
    use std::time::Duration;

    use termion::input::TermRead;
    use voodoo::color::ColorValue;
    use voodoo::terminal::{Mode, Terminal};

    let mut world = specs::World::new();
    components::register_all(&mut world);
    let mut planner = specs::Planner::<()>::new(world, 2);
    planner.add_system(components::map::RenderSystem::new(), "map_render", 10);

    planner.mut_world().create_now().with(components::map::Map::new());

    let (terminal, stdin, mut stdout) = Terminal::new();
    terminal.cursor(Mode::Disabled);
    terminal.clear_color(ColorValue::Black);

    let mut compositor = voodoo::compositor::Compositor::new(80, 24);

    println!("\x1B[?1003h");

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        for event in stdin.events() {
            if let Ok(termion::event::Event::Key(termion::event::Key::Esc)) = event {
                tx.send(event).expect("Couldn't transmit event!");
                break;
            }
            tx.send(event).expect("Couldn't transmit event!");
        }
    });

    let mut last_tick = time::precise_time_ns();
    let mut dt = 0;
    let mut avg_frame_time = 0.0;
    let mut frames: u64 = 0;

    'main: loop {
        for event in rx.try_iter() {
            if let Ok(termion::event::Event::Key(termion::event::Key::Esc)) = event {
                break 'main;
            }
            println!("{:?}", event);
        }

        let now = time::precise_time_ns();
        let old_tick = last_tick;
        dt += now - last_tick;
        last_tick = now;

        while dt > TICK_TIME {
            dt -= TICK_TIME;
            planner.dispatch(());
        }

        // compositor.refresh(&mut stdout);
        thread::sleep(Duration::from_millis((TICK_TIME - dt) / MS));
        let frame_time = time::precise_time_ns() - old_tick;
        avg_frame_time = ((frames as f64 * avg_frame_time) + frame_time as f64) / (frames as f64 + 1.0);
        frames += 1;
    }

    println!("\x1B[?1003l");

    avg_frame_time
}

fn main() {
    let avg_frame_time = run();
    println!("Average frame time: {:03.03} ms", avg_frame_time / MS as f64);
}
