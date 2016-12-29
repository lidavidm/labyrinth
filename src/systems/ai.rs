use std::sync::mpsc;
use specs::{self, Join};

use ::components::{ai, map, player, position};
use ::components::input::OffsetMovable;
use ::util;

pub struct AiSystem {
    ai_begin: mpsc::Receiver<()>,
    ai_end: mpsc::Sender<()>,
}

impl AiSystem {
    pub fn new(ai_begin: mpsc::Receiver<()>, ai_end: mpsc::Sender<()>) -> AiSystem {
        AiSystem {
            ai_begin: ai_begin,
            ai_end: ai_end,
        }
    }
}

impl specs::System<()> for AiSystem {
    fn run(&mut self, arg: specs::RunArg, _: ()) {
        if let Ok(()) = self.ai_begin.try_recv() {

        }
        else {
            // Required to make specs not panic
            arg.fetch(|_| {});
            return;
        }

        let (mut map, mut chase_behaviors, players, mut positions) = arg.fetch(|world| {
            (
                world.write_resource::<map::Map>(),
                world.write::<ai::ChaseBehavior>(),
                world.read::<player::Player>(),
                world.write::<position::Position>(),
            )
        });

        let mut player_position = (0, 0);
        for (_, position) in (&players, &positions).iter() {
            player_position = (position.x, position.y);
        }

        for (chaser, position) in (&mut chase_behaviors, &mut positions).iter() {
            if util::distance2(player_position, (position.x, position.y)) < 25 {
                chaser.spotted = Some(player_position);
            }

            if let Some((x, y)) = chaser.spotted {
                let dx = (x as i32 - position.x as i32).signum();
                let dy = (y as i32 - position.y as i32).signum();

                // Move towards the last known player position if
                // seen, else forget
                if dx != 0 {
                    if let Ok(_) = position.move_by((dx, 0), &mut map) {
                        continue;
                    }
                }
                if dy != 0 {
                    if let Ok(_) = position.move_by((0, dy), &mut map) {
                        continue;
                    }
                }

                chaser.spotted = None;
            }
        }

        self.ai_end.send(()).unwrap();
    }
}