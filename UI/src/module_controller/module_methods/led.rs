use std::sync::Arc;

use pinora_protocol::{LedEvent, LedCommandPayload, ModuleCommand};
use slint::ComponentHandle;

use crate::{AppWindow, LedState, LedUpdates, Unsigned32, type_box::CommandsEventCallback};

impl LedState {
    pub fn new(t_command: CommandsEventCallback, ui: slint::Weak<AppWindow>) -> Self {
         let t_command = Arc::clone(&t_command);

        ui.upgrade_in_event_loop(move |ui| {
            let updates = ui.global::<LedUpdates>();
            updates.on_ledChanged(move |id, value| {
                let d = ModuleCommand::Led(LedCommandPayload::SetState { state: value as u32 });
                let incoming_command = pinora_protocol::IncomingCommand { id: id.into(), command: d };
                t_command(incoming_command);
            });
        })
        .unwrap();
        Self::default()
    }

    pub fn update(&mut self, event: LedEvent ) {
        match event {
            LedEvent::Brightness { id, level } => {
                self.id = id.into();
                self.brightness = level as f32;
            }
        }
    }

    pub fn publish(&self, ui: &slint::Weak<AppWindow>) {
        let id = self.id.clone();
        let key = self.clone();
        let schedule_result = ui.upgrade_in_event_loop(move |app| {
            let updates = app.global::<LedUpdates>();

            updates.set_led_id(id);
            updates.set_led_state(key);

            let revision = updates.get_led_revision();
            updates.set_led_revision(revision.wrapping_add(1));
        });

        if let Err(error) = schedule_result {
            eprintln!("Led UI scheduling failed: {}", error);
        }
    }

  
}
