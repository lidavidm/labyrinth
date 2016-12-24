extern crate rand;
extern crate specs;
extern crate termion;
extern crate time;
extern crate voodoo;

mod components;

use std::sync::mpsc;
use std::thread;

const MS: u64 = 1_000_000;
const TICK_TIME: u64 = MS * 25;

const WIDTH: u16 = 80;
const HEIGHT: u16 = 24;

fn async_events<R: std::io::Read + Send + 'static>(stdin: R) -> mpsc::Receiver<std::io::Result<termion::event::Event>> {
    use termion::input::TermRead;

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

    rx
}

fn run() -> f64 {
    use std::time::Duration;

    use specs::Join;
    use voodoo::color::ColorValue;
    use voodoo::terminal::{Mode, Terminal};
    use voodoo::window::{Point, Window};

    let mut world = specs::World::new();
    components::register_all(&mut world);
    world.add_resource(components::map::Map::new(100, 100));
    let mut planner = specs::Planner::<()>::new(world, 2);
    let (input_system, key_event_channel) = components::input::InputSystem::new();
    planner.add_system(input_system, "input", 100);
    planner.add_system(components::drawable::RenderSystem::new(), "drawable_render", 10);
    planner.add_system(components::map::RenderSystem::new(), "map_render", 10);
    planner.add_system(components::map::BuilderSystem::new(), "map_build", 20);

    planner.mut_world().create_now()
        .with(components::camera::Camera::new((80, 24), (100, 100)))
        .with(components::map::MapRender::new(Window::new(Point::new(0, 0), WIDTH, HEIGHT)))
        .with(components::drawable::DrawableRender::new(voodoo::overlay::Overlay::new(Point::new(0, 0), WIDTH, HEIGHT)))
        .with(components::map::MapBuilder::new());

    planner.mut_world().create_now()
        .with(components::input::Movable)
        .with(components::position::Position { x: 0, y: 0 })
        .with(components::drawable::StaticDrawable { tc: '@'.into() });

    let (terminal, stdin, mut stdout) = Terminal::new();
    terminal.cursor(Mode::Disabled);
    terminal.clear_color(ColorValue::Black);

    let mut compositor = voodoo::compositor::Compositor::new(WIDTH, HEIGHT);

    let rx = async_events(stdin);

    let mut last_tick = time::precise_time_ns();
    let mut dt = 0;
    let mut avg_frame_time = 0.0;
    let mut frames: u64 = 0;

    'main: loop {
        for event in rx.try_iter() {
            if let Ok(termion::event::Event::Key(termion::event::Key::Esc)) = event {
                break 'main;
            }
            else if let Ok(termion::event::Event::Key(k)) = event {
                key_event_channel.send(k).unwrap();
            }
            // TODO: dispatch key events to appropriate systems
        }

        let now = time::precise_time_ns();
        let old_tick = last_tick;
        dt += now - last_tick;
        last_tick = now;

        while dt > TICK_TIME {
            dt -= TICK_TIME;
            planner.dispatch(());
        }

        // TODO: better solution here!
        {
            let maps = planner.mut_world().read::<components::map::MapRender>();
            for map in maps.iter() {
                map.refresh(&mut compositor);
            }
        }
        let drawables = planner.mut_world().read::<components::drawable::DrawableRender>();
        for drawable in drawables.iter() {
            drawable.refresh(&mut compositor);
        }

        compositor.display(&mut stdout);
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
