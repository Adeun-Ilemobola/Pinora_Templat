import { BrowserWindow, BrowserView } from "electrobun/bun";
import type { AppRPC, SerialDeviceInfo } from "../shared/rpc";
import {
  SerialPort,
  list,
  readlineParser,
} from "bun-serialport";
import z from "zod";
import { InComingMessageSchema } from "../shared/Protocol/ModuleDefinitionSchema";
let espPort: any = null;
let start = false;
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
        console.log("Loading serialport...");

        const ports = await list();

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
          // Bun/ElectroBun main process
          mainWindow.webview.rpc?.send.PortStatus({
              path:port,
              status:"connecting"
            })
          espPort = new SerialPort({
            path: port,
            baudRate: 115200,
            autoOpen: false,
          });

          espPort.on("open", () => {
            console.log("Serial port opened");
            mainWindow.webview.rpc?.send.PortStatus({
              path:port,
              status:"connected"
            })
          });

          espPort.on("error", (error: Error) => {
            espPort = null
            console.error("Serial port error:", error);
            mainWindow.webview.rpc?.send.PortStatus({
              path:port,
              status:"error"
            })
          });

          espPort.on("close", () => {
            espPort = null
            console.log("Serial port closed");
            mainWindow.webview.rpc?.send.PortStatus({
              path:port,
              status:"disconnected"
            })
          });

          const parser = espPort.pipe(
            readlineParser({
              delimiter: "\n",
              encoding: "utf-8",
            }),
          );

          parser.on("data", (line: string) => {
            // console.log("ESP32:", line);
            if (line.startsWith("I (422) main_task: Calling app_main()")) {
              start = true;
            }
            if (isValidJSON(line)) {
              const data = JSON.parse(line)
              // console.log("ESP32:", data);
              const parsedMessage = InComingMessageSchema.safeParse(data);

              if (!parsedMessage.success) {
                console.error(
                  "[ListenStore] invalid incoming message:",
                  z.prettifyError(parsedMessage.error),
                );
                return;
              }

              const message = parsedMessage.data;
              // console.log("ESP32:", message);
              mainWindow.webview.rpc?.send.incomingMessage({ message })


            }



          });

          await espPort.open();

          return true;

        } catch (error) {
          console.error(error)
          espPort = null
          mainWindow.webview.rpc?.send.PortStatus({
              path:port,
              status:"error"
            })
          throw new Error("Something went wrong" + error,);


        }


      },

      async sendComand(params) {
        try {
          console.log("new command :", params)
          if (!espPort) {
            throw new Error("no Port available");
          }
          const sendData = `${JSON.stringify(params)}\n`;
          console.log("stringify data :", sendData)
          const result = await espPort.write(sendData);

          console.log("Serial write result:", result);

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