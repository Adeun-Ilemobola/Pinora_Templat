import { SerialPort } from "serialport";
import { createInterface } from "node:readline";

const input = createInterface({
  input: process.stdin,
  crlfDelay: Infinity,
});

let port = null;

// IMPORTANT:
// stdout belongs ONLY to our IPC protocol.
// Debugging/logging must use stderr.
function send(message) {
  process.stdout.write(JSON.stringify(message) + "\n");
}

function log(...args) {
  console.error("[serial-bridge]", ...args);
}

function success(id, result = {}) {
  send({
    type: "response",
    id,
    ok: true,
    result,
  });
}

function failure(id, error) {
  send({
    type: "response",
    id,
    ok: false,
    error: error instanceof Error ? error.message : String(error),
  });
}

function event(event, data = {}) {
  send({
    type: "event",
    event,
    ...data,
  });
}

async function closePort() {
  if (!port) {
    return;
  }

  const currentPort = port;
  port = null;

  if (!currentPort.isOpen) {
    return;
  }

  await new Promise((resolve, reject) => {
    currentPort.close((error) => {
      if (error) {
        reject(error);
        return;
      }

      resolve();
    });
  });
}

async function openPort(path, baudRate) {
  // Only one serial connection belongs to this bridge.
  await closePort();

  const newPort = new SerialPort({
    path,
    baudRate,
    autoOpen: false,
  });

  newPort.on("data", (data) => {
    event("data", {
      data: data.toString("base64"),
    });
  });

  newPort.on("error", (error) => {
    event("error", {
      message: error.message,
    });
  });

  newPort.on("close", () => {
    if (port === newPort) {
      port = null;
    }

    event("closed", {
      path,
    });
  });

  await new Promise((resolve, reject) => {
    newPort.open((error) => {
      if (error) {
        reject(error);
        return;
      }

      resolve();
    });
  });

  port = newPort;
}

async function writePort(base64Data) {
  if (!port?.isOpen) {
    throw new Error("Serial port is not open");
  }

  const data = Buffer.from(base64Data, "base64");

  await new Promise((resolve, reject) => {
    port.write(data, (error) => {
      if (error) {
        reject(error);
        return;
      }

      resolve();
    });
  });

  await new Promise((resolve, reject) => {
    port.drain((error) => {
      if (error) {
        reject(error);
        return;
      }

      resolve();
    });
  });
}

async function handleRequest(message) {
  const { id, type } = message;

  if (id === undefined) {
    throw new Error("Request is missing an id");
  }

  switch (type) {
    case "list": {
      const ports = await SerialPort.list();

      success(id, {
        ports,
      });

      break;
    }

    case "open": {
      await openPort(message.path, message.baudRate);

      success(id, {
        path: message.path,
      });

      event("opened", {
        path: message.path,
      });

      break;
    }

    case "write": {
      await writePort(message.data);

      success(id);

      break;
    }

    case "close": {
      await closePort();

      success(id);

      break;
    }

    default:
      throw new Error(`Unknown request type: ${type}`);
  }
}

input.on("line", async (line) => {
  const text = line.trim();

  if (!text) {
    return;
  }

  let message;

  try {
    message = JSON.parse(text);
  } catch {
    log("Invalid JSON:", text);
    return;
  }

  try {
    await handleRequest(message);
  } catch (error) {
    failure(message.id, error);
  }
});

process.on("SIGINT", async () => {
  try {
    await closePort();
  } finally {
    process.exit(0);
  }
});

process.on("SIGTERM", async () => {
  try {
    await closePort();
  } finally {
    process.exit(0);
  }
});

log("ready");