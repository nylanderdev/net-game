pub struct Velocity {
    velocity: f32,
    angular_velocity: f32,
}

impl Velocity {
    pub fn new(velocity: f32, angular_velocity: f32) -> Self {
        Self {
            velocity,
            angular_velocity,
        }
    }
    pub fn get_velocity(&self) -> f32 {
        self.velocity
    }
    pub fn get_angular_velocity(&self) -> f32 {
        self.angular_velocity
    }
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
    }
    pub fn set_angular_velocity(&mut self, angular_velocity: f32) {
        self.angular_velocity = angular_velocity;
    }
}
