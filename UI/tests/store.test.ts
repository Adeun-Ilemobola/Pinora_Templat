// Characterization tests for the desktop store (`useModuleFront`), with the Tauri
// IPC mocked (audit Phase 2D, PIN-TPL-020). They pin down the behavior the store split
// must keep: connect sequences, message routing, registration, auth status, loss
// handling and the bounded log history.
//
// Run from UI/: `bun test`.
import { beforeEach, describe, expect, mock, test } from "bun:test";

type Listener = (event: { payload: unknown }) => void;
const invoked: { cmd: string; args?: Record<string, unknown> }[] = [];
const listeners = new Map<string, Listener>();

mock.module("@tauri-apps/api/core", () => ({
  invoke: async (cmd: string, args?: Record<string, unknown>) => {
    invoked.push({ cmd, args });
    if (cmd === "get_wifi_listener_status") return { listening: false, connected: false, port: 7878 };
    if (cmd === "start_port") return "Connected";
    return null;
  },
}));
mock.module("@tauri-apps/api/event", () => ({
  listen: async (name: string, callback: Listener) => {
    listeners.set(name, callback);
    return () => listeners.delete(name);
  },
}));

const { useModuleFront } = await import("../src/lib/Modulefront");
const { LOG_CAPACITY } = await import("../src/lib/logs");

const store = () => useModuleFront.getState();
const emit = (name: string, payload: unknown) => {
  const listener = listeners.get(name);
  if (!listener) throw new Error(`no listener for ${name}`);
  listener({ payload });
};
const line = (value: unknown) => JSON.stringify(value);
const flush = () => new Promise((resolve) => setTimeout(resolve, 80));

const registration = (id: string, name: string) => ({
  Registration: { id, module_type: "Led", lool_up_id: name, parent_id: "" },
});
const brightness = (id: string, level: number) => ({
  ModuleEvent: { id, event: { Led: { Brightness: { level } } } },
});

beforeEach(async () => {
  await store().Disconnect();
  store().clearLogs();
  invoked.length = 0;
});

describe("connecting", () => {
  test("Bluetooth: clears state, installs listeners, opens the port, requests rediscovery", async () => {
    await store().ConnectBluetooth("COM7");
    const commands = invoked.map((i) => i.cmd);
    expect(commands).toContain("start_bluetooth_port");
    expect(invoked.find((i) => i.cmd === "start_bluetooth_port")?.args).toEqual({ name: "COM7" });
    const rediscover = invoked.find(
      (i) => i.cmd === "send_data" && String(i.args?.data).includes("Rediscover"),
    );
    expect(rediscover).toBeDefined();
    for (const name of ["espState", "espConsole", "bluetoothConnection", "serialConnection", "wifiConnection", "deviceAuth"]) {
      expect(listeners.has(name)).toBe(true);
    }
    expect(store().PortStat).toBe("Connected");
    expect(store().TransportMode).toBe("Bluetooth");
  });

  test("Serial: opens the port with name and rate and requests rediscovery", async () => {
    await store().Connect("COM3", 115200);
    expect(invoked.find((i) => i.cmd === "start_port")?.args).toEqual({ name: "COM3", rate: 115200 });
    expect(invoked.some((i) => i.cmd === "send_data")).toBe(true);
    expect(store().TransportMode).toBe("Serial");
    expect(store().PortStat).toBe("Connected");
  });
});

describe("incoming messages", () => {
  test("registrations create LED stores, events reach them, logs record everything", async () => {
    await store().ConnectBluetooth("COM7");
    emit("espState", line(registration("id-1", "led1")));
    emit("espState", line(brightness("id-1", 40)));
    await flush();
    const led = store().ModuleRegistry["id-1"];
    expect(led).toBeDefined();
    expect(store().LookUpId["led1"]).toBe("id-1");
    expect((led.getState() as { state: number | null }).state).toBe(40);
    expect(store().logs.length).toBe(2);
  });

  test("re-registration keeps the existing store", async () => {
    await store().ConnectBluetooth("COM7");
    emit("espState", line(registration("id-1", "led1")));
    const first = store().ModuleRegistry["id-1"];
    emit("espState", line(registration("id-1", "led1")));
    expect(store().ModuleRegistry["id-1"]).toBe(first);
  });

  test("duplicate lookup names are recorded", async () => {
    await store().ConnectBluetooth("COM7");
    emit("espState", line(registration("id-1", "led1")));
    emit("espState", line(registration("id-2", "led1")));
    expect(store().DuplicateLookUpIds).toEqual(["led1"]);
  });

  test("messages that fail the schema are dropped", async () => {
    await store().ConnectBluetooth("COM7");
    emit("espState", line({ ModuleEvent: { id: "x", event: { Toaster: {} } } }));
    emit("espState", "not json");
    await flush();
    expect(store().logs.length).toBe(0);
  });

  test("the system snapshot is stored", async () => {
    await store().ConnectBluetooth("COM7");
    const system = {
      System: {
        protocol_version: 2,
        esp_idf_version: "v5.5.3",
        total_heap: "a",
        current_free_heap: "b",
        lowest_free_heap: "c",
        largest_allocation: "d",
        maximum_app_slot: "e",
        flash: "f",
      },
    };
    emit("espState", line(system));
    expect(store().systemInfo?.protocol_version).toBe(2);
  });

  test("the log history stays bounded", async () => {
    await store().ConnectBluetooth("COM7");
    for (let i = 0; i < LOG_CAPACITY + 1_000; i++) emit("espState", line(brightness("id-x", i % 101)));
    await flush();
    expect(store().logs.length).toBe(LOG_CAPACITY);
  });
});

describe("authentication and loss", () => {
  test("deviceAuth payloads update the status; malformed ones are ignored", async () => {
    await store().ConnectBluetooth("COM7");
    emit("deviceAuth", { state: "Authenticated", device_id: "a1b2c3d4e5f6" });
    expect(store().DeviceAuth).toEqual({ state: "Authenticated", device_id: "a1b2c3d4e5f6" });
    emit("deviceAuth", { state: "Paired", device_id: "x", key: "11" });
    expect(store().DeviceAuth?.state).toBe("Authenticated");
  });

  test("pairing invokes the backend command", async () => {
    await store().Connect("COM3", 115200);
    await store().PairDevice();
    await store().UnpairDevice();
    const commands = invoked.map((i) => i.cmd);
    expect(commands).toContain("provision_device_key");
    expect(commands).toContain("clear_device_key");
  });

  test("a lost Bluetooth link shows an error and clears the registry", async () => {
    await store().ConnectBluetooth("COM7");
    emit("espState", line(registration("id-1", "led1")));
    emit("bluetoothConnection", "Disconnected");
    expect(store().PortStat).toBe("Error");
    expect(store().ModuleRegistry).toEqual({});
  });

  test("a lost USB link shows an error", async () => {
    await store().Connect("COM3", 115200);
    emit("serialConnection", "Disconnected");
    expect(store().PortStat).toBe("Error");
  });

  test("disconnecting stops both transports and clears device state", async () => {
    await store().ConnectBluetooth("COM7");
    emit("espState", line(registration("id-1", "led1")));
    invoked.length = 0;
    await store().Disconnect();
    expect(invoked.map((i) => i.cmd)).toEqual(["stop_port", "stop_wifi_listener"]);
    expect(store().ModuleRegistry).toEqual({});
    expect(store().PortStat).toBe("Disconnected");
  });
});
