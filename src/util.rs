use crate::Rect;

pub trait RectExt {
    fn floor(self) -> Self;
}

impl RectExt for Rect {
    fn floor(self) -> Self {
        Self {
            x: self.x.floor(),
            y: self.y.floor()
        }
    }
}