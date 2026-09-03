use pinora_protocol::ServoEvent;

use crate::ServoState;

impl ServoState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, event: ServoEvent) {
        match event {
            ServoEvent::GetAngle { id, angle } => {
                self.id = id.into();
                self.has_angle = true;
                self.angle = angle;
            }
            ServoEvent::GetMinPivot { id, min_pivot } => {
                self.id = id.into();
                self.has_min_pivot = true;
                self.min_pivot = min_pivot;
            }
            ServoEvent::GetMaxPivot { id, max_pivot } => {
                self.id = id.into();
                self.has_max_pivot = true;
                self.max_pivot = max_pivot;
            }
            ServoEvent::GetOffset { id, angle } => {
                self.id = id.into();
                self.has_offset = true;
                self.offset = angle;
            }
        }
    }
}
