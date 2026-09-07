use std::rc::Rc;

use pinora_protocol::SystemInfo;
use slint::{ComponentHandle, VecModel};

use crate::{AppWindow, ModuleView, RemoteReceiverState, RemoteReceiverUpdates};

pub fn publish_module_list(ui: &slint::Weak<AppWindow>, list: Vec<ModuleView>) {
    let _ = ui.upgrade_in_event_loop(move |ui| {
        let model = Rc::new(VecModel::from(list));
        ui.set_module_list(model.into());
    });
}

pub fn publish_system_info(ui: &slint::Weak<AppWindow>, system_info: SystemInfo) {
    let _ = ui.upgrade_in_event_loop(move |ui| {
        ui.set_esp_idf_version(system_info.esp_idf_version.into());
        ui.set_total_heap(system_info.total_heap.into());
        ui.set_current_free_heap(system_info.current_free_heap.into());
        ui.set_lowest_free_heap(system_info.lowest_free_heap.into());
        ui.set_largest_allocation(system_info.largest_allocation.into());
        ui.set_maximum_app_slot(system_info.maximum_app_slot.into());
        ui.set_flash_size(system_info.flash.into());
    });
}

pub fn publish_dashboard_counts(ui: &slint::Weak<AppWindow>, module_count: i32, error_count: i32) {
    let _ = ui.upgrade_in_event_loop(move |ui| {
        ui.set_registered_module_count(module_count);
        ui.set_total_error_count(error_count);
    });
}

