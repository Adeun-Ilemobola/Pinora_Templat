use pinora_protocol::LidarEvent;

use crate::module_controller::module_definition::LidarState;

impl LidarState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, event: LidarEvent) {
        match event {
            LidarEvent::Roi { id, min, max } => {
                self.id = id;
                self.roi_min = Some(min);
                self.roi_max = Some(max);
            }
            LidarEvent::PointMap {
                id,
                max_chunk,
                curr_chunk,
                map,
            } => {
                self.id = id;
                self.max_chunk = Some(max_chunk);
                self.current_chunk = Some(curr_chunk);
                self.map = Some(map);
            }
            LidarEvent::Target { id, point } => {
                self.id = id;
                self.target = Some(point);
            }
            LidarEvent::ScanState {
                id,
                state,
                scan_time,
            } => {
                self.id = id;
                self.scan_state = Some(state);
                self.scan_time = Some(scan_time);
            }
        }
    }
}
