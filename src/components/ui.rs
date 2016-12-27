use specs;

/// This entity has the UI focus.
#[derive(Clone,Copy,Debug,Default,Eq,PartialEq)]
pub struct Focus;

impl specs::Component for Focus {
    type Storage = specs::NullStorage<Focus>;
}
