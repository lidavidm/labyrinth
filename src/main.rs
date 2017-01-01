extern crate rand;
extern crate specs;
extern crate termion;
extern crate time;
extern crate voodoo;

pub mod components;
pub mod systems;
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
    world.add_resource(systems::ui::InfoPanelResource::new(Window::new(Point::new(MAP_WIDTH + 2, 0), 80 - 2 - MAP_WIDTH, 2)));
    world.add_resource(systems::ui::CommandPanelResource::new(Window::new(Point::new(0, MAP_HEIGHT + 2), MAP_WIDTH + 2, 4)));

    let mut map_frame = Window::new(Point::new(0, 0), MAP_WIDTH + 2, MAP_HEIGHT + 2);
    map_frame.border();
    map_frame.print_at(Point::new(1, 0), "MAP");
    let mut msg_frame = Window::new(Point::new(MAP_WIDTH + 2, 2), WIDTH - 2 - MAP_WIDTH, HEIGHT - 2);
    msg_frame.border();
    msg_frame.print_at(Point::new(1, 0), "MESSAGES");
    let y = msg_frame.height - 1;
    msg_frame.print_at(Point::new(1, y), "PgUp/Down—Scroll");
    let point = Point::new(msg_frame.position.x + 1, msg_frame.position.y + 1);
    world.add_resource(systems::ui::MessagesPanelResource::new(Window::new(point, msg_frame.width - 2, msg_frame.height - 2)));

    let mut planner = specs::Planner::<()>::new(world, 2);

    // Setup systems
    let msg_resource = {
        let (sys, res) = systems::ui::MessagesPanelSystem::new();
        planner.add_system(sys, "messages", 1);
        res
    };
    let (ab_tx, ab_rx) = mpsc::channel();
    let (ae_tx, ae_rx) = mpsc::channel();

    let (input_system, key_event_channel) = components::input::InputSystem::new(msg_resource.clone(), ab_tx, ae_rx);
    planner.add_system(input_system, "input", 100);
    planner.add_system(components::drawable::RenderSystem::new(), "drawable_render", 10);
    planner.add_system(components::map::RenderSystem::new(), "map_render", 10);
    planner.add_system(components::map::BuilderSystem::new(msg_resource.clone()), "map_build", 20);
    planner.add_system(systems::ai::AiSystem::new(msg_resource.clone(), ab_rx, ae_tx), "ai", 1);
    let (dead_system, on_player_dead) = systems::ai::DeadSystem::new();
    planner.add_system(dead_system, "dead", 1);
    planner.add_system(systems::combat::CombatSystem::new(msg_resource.clone()), "combat", 100);
    planner.add_system(systems::ui::InfoPanelSystem::new(), "info_panel", 1);

    // Add default entities
    let mut camera = components::camera::Camera::new((MAP_WIDTH, MAP_HEIGHT), (100, 100));
    camera.center_on(50, 50);
    planner.mut_world().create_now()
        .with(camera)
        .with(components::map::MapRender::new(Window::new(Point::new(1, 1), MAP_WIDTH, MAP_HEIGHT)))
        .with(components::drawable::DrawableRender::new(voodoo::overlay::Overlay::new(Point::new(1, 1), MAP_WIDTH, MAP_HEIGHT)))
        .with(components::map::MapBuilder::new());

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

    'main: loop {
        for event in rx.try_iter() {
            if let Ok(termion::event::Event::Key(termion::event::Key::Esc)) = event {
                break 'main;
            }
            else if let Ok(termion::event::Event::Key(k)) = event {
                key_event_channel.send(k).unwrap();
            }
        }

        if let Ok(()) = on_player_dead.try_recv() {
            // TODO: show 'game over'
            break 'main;
        }

        let now = time::precise_time_ns();
        let old_tick = last_tick;
        dt += now - last_tick;
        last_tick = now;

        while dt > TICK_TIME {
            dt -= TICK_TIME;
            planner.dispatch(());
        }

        map_frame.refresh(&mut compositor);
        msg_frame.refresh(&mut compositor);
        let world = planner.mut_world();
        let info = world.read_resource::<systems::ui::InfoPanelResource>();
        let command = world.read_resource::<systems::ui::CommandPanelResource>();
        let messages = world.read_resource::<systems::ui::MessagesPanelResource>();
        let maps = world.read::<components::map::MapRender>();
        let drawables = world.read::<components::drawable::DrawableRender>();
        info.window.refresh(&mut compositor);
        command.window.refresh(&mut compositor);
        messages.window.refresh(&mut compositor);
        for map in maps.iter() {
            map.refresh(&mut compositor);
        }
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
