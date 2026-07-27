// serial-worker.ts — executed with Node
import { SerialPort } from "serialport";
import readline from "node:readline";

const input = readline.createInterface({
  input: process.stdin,
});

let port: SerialPort | undefined;

input.on("line", line => {
  try {
    const message = JSON.parse(line);

    if (message.type === "open") {
      port = new SerialPort({
        path: message.path,
        baudRate: message.baudRate,
      });

      port.on("open", () => {
        send({ type: "opened", path: message.path });
      });

      port.on("data", data => {
        send({
          type: "data",
          data: data.toString("base64"),
        });
      });

      port.on("error", error => {
        send({
          type: "error",
          message: error.message,
        });
      });
    }
  } catch (error) {
    send({
      type: "error",
      message: error instanceof Error ? error.message : String(error),
    });
  }
});

function send(message: unknown) {
  process.stdout.write(JSON.stringify(message) + "\n");
}