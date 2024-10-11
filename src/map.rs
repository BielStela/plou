include!(concat!(env!("OUT_DIR"), "/coordinates.rs"));


use ratatui::style::Color;
use ratatui::widgets::canvas::{Painter, Shape};

#[derive(Debug, Clone, Default, Copy, Eq, PartialEq, Hash)]
pub enum WorldResolution {
    #[default]
    Low,
    Med,
    High,
}

impl WorldResolution {
    const fn data(self) -> &'static [(f64, f64)] {
        &COORDINATES
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct WorldMap {
    pub resolution: WorldResolution,
    pub color: Color,
}

impl Shape for WorldMap {
    fn draw(&self, painter: &mut Painter) {
        for (x, y) in self.resolution.data() {
            if let Some((x, y)) = painter.get_point(*x, *y) {
                painter.paint(x, y, self.color);
            }
        }
    }
}
