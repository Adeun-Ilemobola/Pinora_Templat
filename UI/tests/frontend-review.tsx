// Local browser fixture: sample protocol traffic only; no transport is connected.
import { createRoot } from "react-dom/client";

import App from "../src/App";
import { useModuleFront } from "../src/lib/Modulefront";
import { recordLogReceipt } from "../src/lib/logs";
import type { ProtocolMessage } from "../src/lib/protocol/message";

const logs: ProtocolMessage[] = [{
  Registration: {
    id: "servo-1",
    module_type: "Servo",
    lool_up_id: "servo_x",
    parent_id: "lidar"
  }
}, ...(["Low", "Medium", "High", "Critical"] as const).map(priority => ({
  ModuleEvent: {
    id: "rangefinder",

    event: {
      SysLog: {
        priority,
        text: `${priority} sensor diagnostic`,
        raw_err: "Measurement timeout: retry exhausted"
      }
    }
  }
})), {
  System: {
    esp_idf_version: "5.4",
    total_heap: "1000",
    current_free_heap: "500",
    lowest_free_heap: "400",
    largest_allocation: "200",
    maximum_app_slot: "10000",
    flash: "20000"
  }
}];

logs.forEach(recordLogReceipt);

useModuleFront.setState({
  logs
});

for (const [id, module_type] of [["lidar", "Lidar"], ["servo_x", "Servo"], ["servo_y", "Servo"], ["rangefinder", "Rangefinder"]] as const) {
  useModuleFront.getState().RegistrationEvent({id, module_type, lool_up_id: id, parent_id: module_type === "Lidar" ? "" : "lidar"});
}
createRoot(document.getElementById("root")!).render(<App />);
