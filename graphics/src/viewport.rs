#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct Viewport<T> {
    x: T,
    y: T,
    width: T,
    height: T,
}

impl<T> Viewport<T>
where
    T: Copy,
{
    pub fn new(x: T, y: T, width: T, height: T) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn x(&self) -> T {
        self.x
    }

    pub fn y(&self) -> T {
        self.y
    }

    pub fn width(&self) -> T {
        self.width
    }

    pub fn height(&self) -> T {
        self.height
    }

    pub fn position(&self) -> (T, T) {
        (self.x, self.y)
    }

    pub fn dimensions(&self) -> (T, T) {
        (self.width, self.height)
    }
}
