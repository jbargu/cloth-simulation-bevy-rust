#[derive(Debug)]
pub struct Node {
    // Current position
    pub x: i16,
    pub y: i16,

    // Previous position
    pub prev_x: i16,
    pub prev_y: i16,

    // Acceleration
    pub ax: i16,
    pub ay: i16,

    pinned: bool,
}
impl Node {
    pub fn new(x: i16, y: i16, pinned: bool) -> Node {
        Node {
            x,
            y,
            prev_x: x,
            prev_y: y,
            ax: 0,
            ay: 1,
            pinned,
        }
    }

    pub fn update(&mut self, dt: i16) {
        if self.pinned {
            return;
        }
        let vx = self.x - self.prev_x;
        let vy = self.y - self.prev_y;

        let next_x = self.x + vx + self.ax * dt;
        let next_y = self.y + vy + self.ay * dt;

        self.prev_x = self.x;
        self.prev_y = self.y;

        self.x = next_x;
        self.y = next_y;
    }
}
