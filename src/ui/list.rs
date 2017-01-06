use voodoo::window::Point;

pub struct List<T> {
    pub contents: Vec<T>,
    pub cursor: usize,
    pub bounds: (Point, Point),
    pub normal: super::ColorPair,
    pub highlight: super::ColorPair,
}

pub trait ListRenderable {
    fn render(&self) -> Vec<String>;
}

impl<T: ListRenderable> List<T> {

}
