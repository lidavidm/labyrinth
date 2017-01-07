use voodoo::color::ColorValue;

pub mod list;

pub use self::list::{List, ListRenderable};

pub struct ColorPair {
    pub fg: ColorValue,
    pub bg: ColorValue,
}

impl ColorPair {
    pub fn new(fg: ColorValue, bg: ColorValue) -> ColorPair {
        ColorPair {
            fg: fg,
            bg: bg,
        }
    }
}
