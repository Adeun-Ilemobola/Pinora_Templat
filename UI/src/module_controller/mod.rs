pub mod module_definition;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use pinora_protocol::{
    ButtonEvent, ImuEvent, LedEvent, LogPriority, ModuleEvent, ModuleType, ProtocolMessage,
    RemoteButtonEvent, RfidEvent, StepperMotorEvent, SystemInfo,
};
use slint::ComponentHandle;

use crate::ModuleTypeS;
use crate::module_controller::module_definition::{
    ButtonState, ImuState, LedState, LidarState, ModuleState, RangefinderState,
    RemoteReceiverState, RfidState, ServoState, StepperMotorState, SysLogState,
};
use crate::{AppWindow, transport::transport_gate::Transport};

pub struct ModuleController {
    link: Arc<Mutex<Transport>>,
    collections: HashMap<String, ModuleState>,
    registered_module_ids: HashSet<String>,
    system_info: Option<SystemInfo>,
    total_errors: u32,
    ui: slint::Weak<AppWindow>,
}

impl ModuleController {
    pub fn new(link: Arc<Mutex<Transport>>, ui: &AppWindow) -> Self {
        ModuleController {
            link: Arc::clone(&link),
            collections: HashMap::new(),
            registered_module_ids: HashSet::new(),
            system_info: None,
            total_errors: 0,
            ui: ui.as_weak(),
        }
    }

    pub fn incoming_event(&mut self, data: Vec<u8>) {
        let Ok(message) = serde_json::from_slice::<ProtocolMessage>(&data) else {
            println!(
                "Ignoring non-protocol data: {}",
                String::from_utf8_lossy(&data)
            );
            return;
        };

        match message {
            ProtocolMessage::Registration(registration) => {
                let module_id = registration.id;
                let state = match registration.module_type {
                    ModuleType::Servo => Some(ModuleState::Servo(ServoState::new())),
                    ModuleType::Led => Some(ModuleState::Led(LedState::new())),
                    ModuleType::Imu => Some(ModuleState::Imu(ImuState::new())),
                    ModuleType::Button => Some(ModuleState::Button(ButtonState::new())),
                    ModuleType::Lidar => Some(ModuleState::Lidar(LidarState::new())),
                    ModuleType::Rangefinder => {
                        Some(ModuleState::Rangefinder(RangefinderState::new()))
                    }
                    ModuleType::SysLog => Some(ModuleState::SysLog(SysLogState::new())),
                    ModuleType::StepperMotor => {
                        Some(ModuleState::StepperMotor(StepperMotorState::new()))
                    }
                    ModuleType::Rfid => Some(ModuleState::Rfid(RfidState::new())),
                    ModuleType::RemoteReceiver => {
                        Some(ModuleState::RemoteReceiver(RemoteReceiverState::new()))
                    }
                    ModuleType::LedCluster | ModuleType::JoyStick => None,
                };

                if let Some(state) = state {
                    self.collections.insert(module_id.clone(), state);
                }
                self.registered_module_ids.insert(module_id);
                self.publish_dashboard_counts();
            }
            ProtocolMessage::System(system_info) => {
                let system_info_for_ui = system_info.clone();
                let _ = self.ui.upgrade_in_event_loop(move |ui| {
                    ui.set_esp_idf_version(system_info_for_ui.esp_idf_version.into());
                    ui.set_total_heap(system_info_for_ui.total_heap.into());
                    ui.set_current_free_heap(system_info_for_ui.current_free_heap.into());
                    ui.set_lowest_free_heap(system_info_for_ui.lowest_free_heap.into());
                    ui.set_largest_allocation(system_info_for_ui.largest_allocation.into());
                    ui.set_maximum_app_slot(system_info_for_ui.maximum_app_slot.into());
                    ui.set_flash_size(system_info_for_ui.flash.into());
                });
                self.system_info = Some(system_info);
            }
            ProtocolMessage::ModuleEvent(event) => {
                if matches!(
                    &event,
                    ModuleEvent::SysLog(log)
                        if matches!(&log.priority, LogPriority::High | LogPriority::Critical)
                ) {
                    self.total_errors = self.total_errors.saturating_add(1);
                    self.publish_dashboard_counts();
                }
                self.handle_state_change(event);
            }
        }
    }

    fn handle_state_change(&mut self, event: ModuleEvent) {
        if let Some(id) = Self::module_event_id(&event).map(str::to_owned) {
            if self.module_exists(&id) {
                if let Some(state) = self.collections.get_mut(&id) {
                    state.update(event);
                }
            }
        }
    }

    fn module_exists(&self, id: &str) -> bool {
        self.collections.contains_key(id)
    }

    fn publish_dashboard_counts(&self) {
        let module_count = self.registered_module_ids.len().min(i32::MAX as usize) as i32;
        let error_count = self.total_errors.min(i32::MAX as u32) as i32;

        let _ = self.ui.upgrade_in_event_loop(move |ui| {
            ui.set_registered_module_count(module_count);
            ui.set_total_error_count(error_count);
        });
    }

    fn module_event_id(event: &ModuleEvent) -> Option<&str> {
        match event {
            ModuleEvent::Led(LedEvent::Brightness { id, .. }) => Some(id),
            ModuleEvent::Button(ButtonEvent::Ckick { id }) => Some(id),
            ModuleEvent::SysLog(_) => None,
            ModuleEvent::RemoteReceiver(RemoteButtonEvent::Click { id, .. }) => Some(id),
            ModuleEvent::StepperMotor(event) => match event {
                StepperMotorEvent::GetAngle { id, .. }
                | StepperMotorEvent::GetPivotMin { id, .. }
                | StepperMotorEvent::GetPivotMax { id, .. }
                | StepperMotorEvent::GetMode { id, .. }
                | StepperMotorEvent::GetOrigin { id, .. }
                | StepperMotorEvent::GetPivotPoint { id, .. } => Some(id),
            },
            ModuleEvent::Imu(event) => match event {
                ImuEvent::Gyro { id, .. } | ImuEvent::Accel { id, .. } => Some(id),
                ImuEvent::Mode { .. } => None,
            },
            ModuleEvent::Rfid(event) => match event {
                RfidEvent::GetCard { id, .. }
                | RfidEvent::GetMode { id, .. }
                | RfidEvent::GetWriteState { id, .. } => Some(id),
            },
        }
    }

    fn module_type_to_slint(ty: ModuleType) -> ModuleTypeS {
        match ty {
            ModuleType::Servo => ModuleTypeS::Servo,
            ModuleType::Led => ModuleTypeS::Led,
            ModuleType::Imu => ModuleTypeS::Imu,
            ModuleType::LedCluster => ModuleTypeS::LedCluster,
            ModuleType::Button => ModuleTypeS::Button,
            ModuleType::Lidar => ModuleTypeS::Lidar,
            ModuleType::Rangefinder => ModuleTypeS::Rangefinder,
            ModuleType::SysLog => ModuleTypeS::SysLog,
            ModuleType::JoyStick => ModuleTypeS::JoyStick,
            ModuleType::StepperMotor => ModuleTypeS::StepperMotor,
            ModuleType::Rfid => ModuleTypeS::Rfid,
            ModuleType::RemoteReceiver => ModuleTypeS::RemoteReceiver,
        }
    }
}
