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

use std::sync::mpsc;

use specs::{self, Join};
use termion;
use voodoo::compositor::Compositor;
use voodoo::overlay::Overlay;
use voodoo::window::{Window, Point};

use ::{WIDTH, HEIGHT, MAP_WIDTH, MAP_HEIGHT};
use ::{components, systems};

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum SubGameScreen {
    Map,
    Inventory,
}

pub struct GameScreen {
    sub_screen: Vec<SubGameScreen>,
    map_frame: Window,
    msg_frame: Window,
    event_channel: mpsc::Sender<components::input::Event>,
    sub_screen_channel: mpsc::Receiver<super::SubScreenEvent<SubGameScreen>>,
}

impl super::Screen for GameScreen {
    fn setup(planner: &mut specs::Planner<()>, transitions: super::TransitionChannel) -> GameScreen {
        let (sub_screen_sender, sub_screen_channel) = mpsc::channel();

        let (map_frame, msg_frame) = {
            let world = planner.mut_world();
            world.add_resource(components::map::Map::new(100, 100));
            world.add_resource(systems::ui::InfoPanelResource::new(
                Window::new(Point::new(MAP_WIDTH + 2, 0), 80 - 2 - MAP_WIDTH, 2)));
            world.add_resource(systems::ui::CommandPanelResource::new(
                Window::new(Point::new(0, MAP_HEIGHT + 2), MAP_WIDTH + 2, 4)));

            let mut map_frame = Window::new(Point::new(0, 0), MAP_WIDTH + 2, MAP_HEIGHT + 2);
            map_frame.border();
            map_frame.print_at(Point::new(1, 0), "MAP");
            let mut msg_frame = Window::new(Point::new(MAP_WIDTH + 2, 2), WIDTH - 2 - MAP_WIDTH, HEIGHT - 2);
            let y = msg_frame.height - 1;
            msg_frame.print_at(Point::new(1, y), "PgUp/Down—Scroll");
            let point = Point::new(msg_frame.position.x + 1, msg_frame.position.y + 1);
            world.add_resource(systems::ui::MessagesPanelResource::new(
                Window::new(point, msg_frame.width - 2, msg_frame.height - 2)));
            world.add_resource(systems::ui::InventoryPanelResource::new(
                Window::new(point, msg_frame.width - 2, msg_frame.height - 2)));

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

        let (input_system, event_channel) = components::input::InputSystem::new(
            ::ui::List::new(Point::new(0, 0), msg_frame.width - 2, msg_frame.height - 2),
            transitions.clone(), sub_screen_sender, msg_resource.clone(), ab_tx, ae_rx);
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
            sub_screen: vec![SubGameScreen::Map],
            map_frame: map_frame,
            msg_frame: msg_frame,
            event_channel: event_channel,
            sub_screen_channel: sub_screen_channel,
        }
    }

    fn dispatch(&mut self, event: termion::event::Event) {
        self.event_channel.send(match event {
            termion::event::Event::Key(k) => components::input::Event::Key(k),
            termion::event::Event::Mouse(termion::event::MouseEvent::Hold(x, y)) => {
                // Convert to zero-based
                let x = x - 1;
                let y = y - 1;

                if x >= 1 && x < 1 + MAP_WIDTH && y >= 1 && y <= 1 + MAP_HEIGHT {
                    // Convert to relative to map
                    components::input::Event::MouseHover(Point::new(x - 1, y - 1))
                }
                else {
                    return;
                }
            },
            termion::event::Event::Mouse(termion::event::MouseEvent::Release(x, y)) => {
                // Convert to zero-based
                let x = x - 1;
                let y = y - 1;

                if x >= 1 && x < 1 + MAP_WIDTH && y >= 1 && y <= 1 + MAP_HEIGHT {
                    // Convert to relative to map
                    components::input::Event::MouseRelease(Point::new(x - 1, y - 1))
                }
                else {
                    return;
                }
            },
            _ => return,
        }).unwrap();
    }

    fn render(&mut self, planner: &mut specs::Planner<()>, compositor: &mut Compositor) {
        super::SubScreenEvent::apply_all(&self.sub_screen_channel, &mut self.sub_screen);

        self.map_frame.refresh(compositor);
        let world = planner.mut_world();
        let info = world.read_resource::<systems::ui::InfoPanelResource>();
        let command = world.read_resource::<systems::ui::CommandPanelResource>();
        let maps = world.read::<components::map::MapRender>();
        let drawables = world.read::<components::drawable::DrawableRender>();
        info.window.refresh(compositor);
        command.window.refresh(compositor);

        match self.sub_screen.last() {
            Some(&SubGameScreen::Map) | None => {
                let messages = world.read_resource::<systems::ui::MessagesPanelResource>();
                self.msg_frame.border();
                self.msg_frame.print_at(Point::new(1, 0), "MESSAGES");
                self.msg_frame.refresh(compositor);
                messages.window.refresh(compositor);
            }
            Some(&SubGameScreen::Inventory) => {
                let res = world.read_resource::<systems::ui::InventoryPanelResource>();
                self.msg_frame.border();
                self.msg_frame.print_at(Point::new(1, 0), "INVENTORY");
                self.msg_frame.refresh(compositor);
                res.window.refresh(compositor);
            }
        }

        for map in maps.iter() {
            map.refresh(compositor);
        }
        for drawable in drawables.iter() {
            drawable.refresh(compositor);
        }
    }
}
