use std::sync::mpsc;

use specs::{self, Join};
use termion;
use voodoo::compositor::Compositor;
use voodoo::overlay::Overlay;
use voodoo::window::{Window, Point};

use ::{WIDTH, HEIGHT, MAP_WIDTH, MAP_HEIGHT};
use ::{components, systems};

pub struct GameScreen {
    map_frame: Window,
    msg_frame: Window,
    key_event_channel: mpsc::Sender<termion::event::Key>,
}

impl super::Screen for GameScreen {
    fn setup(planner: &mut specs::Planner<()>, transitions: super::TransitionChannel) -> GameScreen {
        let (map_frame, msg_frame) = {
            let world = planner.mut_world();
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

            (map_frame, msg_frame)
        };

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
        planner.add_system(systems::ai::DeadSystem::new(transitions.clone()), "dead", 1);
        planner.add_system(systems::combat::CombatSystem::new(msg_resource.clone()), "combat", 100);
        planner.add_system(systems::ui::InfoPanelSystem::new(), "info_panel", 1);

        // Add default entities
        let mut camera = components::camera::Camera::new((MAP_WIDTH, MAP_HEIGHT), (100, 100));
        camera.center_on(50, 50);
        planner.mut_world().create_now()
            .with(camera)
            .with(components::map::MapRender::new(Window::new(Point::new(1, 1), MAP_WIDTH, MAP_HEIGHT)))
            .with(components::drawable::DrawableRender::new(Overlay::new(Point::new(1, 1), MAP_WIDTH, MAP_HEIGHT)))
            .with(components::map::MapBuilder::new());

        GameScreen {
            map_frame: map_frame,
            msg_frame: msg_frame,
            key_event_channel: key_event_channel,
        }
    }

    fn dispatch(&mut self, event: termion::event::Key) {
        self.key_event_channel.send(event).unwrap();
    }

    fn render(&mut self, planner: &mut specs::Planner<()>, compositor: &mut Compositor) {
        self.map_frame.refresh(compositor);
        self.msg_frame.refresh(compositor);
        let world = planner.mut_world();
        let info = world.read_resource::<systems::ui::InfoPanelResource>();
        let command = world.read_resource::<systems::ui::CommandPanelResource>();
        let messages = world.read_resource::<systems::ui::MessagesPanelResource>();
        let maps = world.read::<components::map::MapRender>();
        let drawables = world.read::<components::drawable::DrawableRender>();
        info.window.refresh(compositor);
        command.window.refresh(compositor);
        messages.window.refresh(compositor);
        for map in maps.iter() {
            map.refresh(compositor);
        }
        for drawable in drawables.iter() {
            drawable.refresh(compositor);
        }
    }
}
