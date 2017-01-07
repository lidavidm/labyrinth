use voodoo::color::ColorValue;
use voodoo::window::{Point, Window};

use super::ColorPair;

pub struct List<T> {
    pub contents: Vec<T>,
    pub cursor: usize,
    pub position: Point,
    pub bounds: (u16, u16),
    pub normal: ColorPair,
    pub highlight: ColorPair,
}

pub trait ListRenderable {
    fn render(&self) -> Vec<String>;
}

impl<T: ListRenderable> List<T> {
    pub fn new(position: Point, width: u16, height: u16) -> List<T> {
        // TODO: check that we have enough height to render everything
        List {
            contents: Vec::new(),
            cursor: 0,
            position: position,
            bounds: (position.x + width, position.y + height),
            normal: ColorPair::new(ColorValue::White, ColorValue::Black),
            highlight: ColorPair::new(ColorValue::Black, ColorValue::White),
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_cursor_down(&mut self) {
        if self.cursor < self.contents.len() - 1 {
            self.cursor += 1;
        }
    }

    pub fn get_selected(&self) -> Option<&T> {
        self.contents.get(self.cursor)
    }

    pub fn refresh(&self, window: &mut Window) {
        // TODO: account for our width
        if self.cursor > 0 {
            window.print_at(self.position, "...");
        }

        let mut rendered = 1;
        let mut early_break = false;
        for item in &self.contents[self.cursor..self.contents.len()] {
            let desc = item.render();
            if rendered + desc.len() >= self.bounds.1 as usize {
                early_break = true;
                break;
            }

            for line in desc {
                window.print_at(Point::new(self.position.x, self.position.y + rendered as u16), &line);
                rendered += 1;
            }
        }

        if early_break {
            window.print_at(Point::new(self.position.x, self.position.y + rendered as u16), "...");
        }
    }
}
