use libtetris::CellColor;
use nannou::Draw;

use crate::wgpu::Texture;

pub enum Skin {
    Basic(Texture)
}

impl Skin {
    pub fn draw_mino(&self, draw: &Draw, piece: CellColor, x: f32, y: f32, size: f32) {
        let Skin::Basic(texture) = &self;
        let Some(index) = color_to_tex_index(piece) else { return; };

        let tx = (1. / 12.) * index as f32;
        const TW: f32 = 1. / 13.;

        let points = [
            ((x, y, 0.), (tx, 1.)), // BL
            ((x + size, y, 0.), (tx + TW, 1.)), // BR
            ((x + size, y + size, 0.), (tx + TW, 0.)), // TR

            ((x, y, 0.), (tx, 1.)), // BL
            ((x + size, y + size, 0.), (tx + TW, 0.)), // TR
            ((x, y + size, 0.), (tx, 0.)), // TL
        ];
        draw.mesh()
            .points_textured(&texture, points);
    }
}

#[inline]
fn color_to_tex_index(color: CellColor) -> Option<usize> {
    match color {
        CellColor::I => Some(4),
        CellColor::O => Some(2),
        CellColor::T => Some(6),
        CellColor::L => Some(1),
        CellColor::J => Some(5),
        CellColor::S => Some(3),
        CellColor::Z => Some(0),
        CellColor::Garbage => Some(9),
        // shh, don't tell anyone, but we don't really use unclearable at any point,
        // so we're using it as the ghost piece here
        CellColor::Unclearable => Some(7),
        CellColor::Empty => None
    }
}