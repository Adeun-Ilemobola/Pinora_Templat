use pinora_protocol::ServoEvent;

use crate::ServoState;

impl ServoState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, id: &str, event: ServoEvent) {
        self.id = id.into();
        match event {
            ServoEvent::GetAngle { angle } => {
                self.has_angle = true;
                self.angle = angle;
            }
            ServoEvent::GetMinPivot { min_pivot } => {
                self.has_min_pivot = true;
                self.min_pivot = min_pivot;
            }
            ServoEvent::GetMaxPivot { max_pivot } => {
                self.has_max_pivot = true;
                self.max_pivot = max_pivot;
            }
            ServoEvent::GetOffset { angle } => {
                self.has_offset = true;
                self.offset = angle;
            }
        }
    }
}
