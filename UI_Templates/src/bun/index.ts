import {
    BrowserWindow,
    BrowserView,
   
} from "electrobun/main";
import type { AppRPC, SerialDeviceInfo } from "@src/bun/rpc";
import z, { any } from "zod";
import { InComingMessageSchema } from "@src/bun/Protocol/ModuleDefinitionSchema";
import { BunSerial } from "@src/bun/Runtime/serial/BunSerial";

const serial = new BunSerial({
  bridgePath:
    "C:/Dev/Pinora/Pinora_Templat/UI_Templates/sidecars/serial/index.mjs",
});

serial.on("opened", (data) => {
  console.log("Serial port opened:", data);
});

serial.on("data", (data) => {

  
  console.log(
    "ESP32:",
    new TextDecoder().decode(data),
  );
});

serial.on("error", (error) => {
  console.error("Serial error:", error);
});

serial.on("closed", (data) => {
  console.log("Serial port closed:", data);
});


const DEV_SERVER_URL = "http://localhost:5173";
function isValidJSON(text: string) {
  try {
    JSON.parse(text);
    return true;
  } catch (error) {
    return false;
  }
}

async function getMainViewUrl(): Promise<string> {
  try {
    const response = await fetch(DEV_SERVER_URL, {
      method: "HEAD",
      signal: AbortSignal.timeout(1000),
    });

    if (response.ok) {
      console.log(`HMR enabled: ${DEV_SERVER_URL}`);
      return DEV_SERVER_URL;
    }
  } catch {
    console.log("Vite server unavailable; using bundled view.");
  }

  return "views://mainview/index.html";
}

export const rpc = BrowserView.defineRPC<AppRPC>({
  handlers: {
    requests: {
      async getAvailablePorts(): Promise<SerialDeviceInfo[]> {

        const ports = await serial.list();


        return ports.map((port: any) => ({
          path: port.path,
          manufacturer: port.manufacturer ?? null,
          serialNumber: port.serialNumber ?? null,
          vendorId: port.vendorId ?? null,
          productId: port.productId ?? null,
          locationId: port.locationId ?? null,
          pnpId: port.pnpId ?? null,
        }));
      },
      async openPort({ port }) {
        try {
          await serial.open({
            path: port,
            baudRate: 115200,
          });





          return true;

        } catch (error) {
          console.error(error)
          mainWindow.webview.rpc?.send.PortStatus({
            path: port,
            status: "error"
          })
          throw new Error("Something went wrong" + error,);


        }
      },

      async sendComand(params) {
        try {


        } catch (error) {
          console.error("Command write failed:", error);
          throw error;
        }

      },
    },
    messages: {},
  },
});

console.log("Resolving main view URL...");

const url = await getMainViewUrl();

console.log("Creating window with URL:", url);

const mainWindow = new BrowserWindow({
  title: "UI_Templates",
  url,
  frame: {
    width: 1000,
    height: 720,
    x: 200,
    y: 200,
  },
  rpc,
});

console.log("UI_Templates window created.");