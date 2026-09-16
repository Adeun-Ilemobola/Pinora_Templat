use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::modem::Modem,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};

use pinora_protocol::registration::ProtocolMessage;
pub struct Wifi {
    wifi: BlockingWifi<EspWifi<'static>>,
}

impl Wifi {
    pub fn new(
        sys_loop: EspSystemEventLoop,
        nvs: EspDefaultNvsPartition,
        modem: Modem<'static>,
    ) -> Result<Self, esp_idf_svc::sys::EspError> {
        let esp_wifi = EspWifi::new(
            modem,
            sys_loop.clone(),
            Some(nvs),
        )?;

        let wifi = BlockingWifi::wrap(
            esp_wifi,
            sys_loop,
        )?;

        Ok(Self {
            wifi,
        })
    }

    pub fn event(&self, event: ProtocolMessage) {
        // Later: send this through the network transport
    }
}