use pinora_protocol::LidarEvent;

use crate::module_controller::module_definition::LidarState;

impl LidarState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, id: &str, event: LidarEvent) {
        self.id = id.to_owned();
        match event {
            LidarEvent::Roi { min, max } => {
                self.roi_min = Some(min);
                self.roi_max = Some(max);
            }
            LidarEvent::PointMap {
                max_chunk,
                curr_chunk,
                map,
            } => {
                self.max_chunk = Some(max_chunk);
                self.current_chunk = Some(curr_chunk);
                self.map = Some(map);
            }
            LidarEvent::Target { point } => {
                self.target = Some(point);
            }
            LidarEvent::ScanState {
                state,
                scan_time,
            } => {
                self.scan_state = Some(state);
                self.scan_time = Some(scan_time);
            }
        }
    }
}
