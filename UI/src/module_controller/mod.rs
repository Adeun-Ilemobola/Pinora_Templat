pub mod module_definition;
mod module_methods;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use pinora_protocol::{
    EventPackage, LogPriority, ModuleEvent, ModuleType, ProtocolMessage,
};
use slint::ComponentHandle;

use crate::AppWindow;
use crate::module_controller::module_definition::{
    ButtonState, ImuState, LedState, LidarState, ModuleState, RangefinderState,
    RemoteReceiverState, RfidState, ServoState, StepperMotorState, SysLogState,
};
use crate::type_box::CommandsEventCallback;
use crate::ui_bridge::publication::{
    publish_dashboard_counts, publish_module_list, publish_system_info,
};
use crate::{ModuleView, UiModuleType};

pub struct ModuleController {
    collections: HashMap<String, ModuleState>,
    registered_module_ids: HashSet<String>,
    total_errors: u32,
    ui: slint::Weak<AppWindow>,
    t_command: CommandsEventCallback,
}

impl ModuleController {
    pub fn new(ui: &AppWindow, t_command: CommandsEventCallback) -> Self {
        ModuleController {
            collections: HashMap::new(),
            registered_module_ids: HashSet::new(),
            total_errors: 0,
            ui: ui.as_weak(),
            t_command,
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
                    ModuleType::Led => Some(ModuleState::Led(LedState::new(
                        Arc::clone(&self.t_command),
                        self.ui.clone(),
                    ))),
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
                    ModuleType::RemoteReceiver => Some(ModuleState::RemoteReceiver(
                        RemoteReceiverState::new(Arc::clone(&self.t_command), self.ui.clone()),
                    )),
                    ModuleType::LedCluster | ModuleType::JoyStick => None,
                };

                if let Some(state) = state {
                    self.collections.insert(module_id.clone(), state);
                    self.registered_module_ids.insert(module_id);
                    self.publish_dashboard_counts();
                    publish_module_list(&self.ui, self.build_ui_module());
                   
                }
            }
            ProtocolMessage::System(system_info) => {
                publish_system_info(&self.ui, system_info);
            }
            ProtocolMessage::ModuleEvent(package) => {
                let EventPackage { id, event } = package;
                if let ModuleEvent::SysLog(log) = event {
                    // Diagnostic identity is the source UUID, even before registration.
                    eprintln!("[module {id}] {:?}: {} {:?}", log.priority, log.text, log.raw_err);
                    if matches!(log.priority, LogPriority::High | LogPriority::Critical) {
                        self.total_errors = self.total_errors.saturating_add(1);
                        self.publish_dashboard_counts();
                    }
                } else {
                    self.apply_module_event(&id, event);
                }
            }
        }
    }

    fn apply_module_event(&mut self, module_id: &str, event: ModuleEvent) {
        let Some(state) = self.collections.get_mut(module_id) else {
            return;
        };

        match (state, event) {
            (ModuleState::Led(state), ModuleEvent::Led(event)) => {
                state.update(module_id, event);
                state.publish(&self.ui);
            }
            (ModuleState::Button(state), ModuleEvent::Button(event)) => state.update(module_id, event),
            (ModuleState::RemoteReceiver(state), ModuleEvent::RemoteReceiver(event)) => {
                state.update(module_id, event);
                state.publish(&self.ui);
            }
            (ModuleState::StepperMotor(state), ModuleEvent::StepperMotor(event)) => {
                state.update(module_id, event);
            }
            (ModuleState::Imu(state), ModuleEvent::Imu(event)) => state.update(module_id, event),
            (ModuleState::Rfid(state), ModuleEvent::Rfid(event)) => state.update(module_id, event),
            _ => {}
        }
    }

    fn publish_dashboard_counts(&self) {
        let module_count = self.registered_module_ids.len().min(i32::MAX as usize) as i32;
        let error_count = self.total_errors.min(i32::MAX as u32) as i32;

        publish_dashboard_counts(&self.ui, module_count, error_count);
    }

    pub fn build_ui_module(&self) -> Vec<ModuleView> {
        let mut ui_list = Vec::new();

        for (id, module) in &self.collections {
            let ui_module = ModuleView {
                module_type: match module {
                    ModuleState::Servo(_) => UiModuleType::Servo,
                    ModuleState::Led(_) => UiModuleType::Led,
                    ModuleState::Imu(_) => UiModuleType::Imu,
                    ModuleState::LedCluster(_) => UiModuleType::LedCluster,
                    ModuleState::Button(_) => UiModuleType::Button,
                    ModuleState::Lidar(_) => UiModuleType::Lidar,
                    ModuleState::Rangefinder(_) => UiModuleType::Rangefinder,
                    ModuleState::SysLog(_) => UiModuleType::SysLog,
                    ModuleState::JoyStick(_) => UiModuleType::JoyStick,
                    ModuleState::StepperMotor(_) => UiModuleType::StepperMotor,
                    ModuleState::Rfid(_) => UiModuleType::Rfid,
                    ModuleState::RemoteReceiver(_) => UiModuleType::RemoteReceiver,
                },

                id: id.into(),
            };

            ui_list.push(ui_module);
        }

        for m in ui_list.iter() {
            println!("UI module: {:?}", m);
        }

        ui_list
    }
}

#[cfg(test)]
mod tests;
