use specs;

/// This entity is the player.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct Player;

impl specs::Component for Player {
    type Storage = specs::NullStorage<Player>;
}
