// The frontend Zod schemas must accept every line the Rust protocol crate produces
// (Phase 2A, PIN-TPL-007). The lines come from the Rust golden fixture, which
// protocol/tests/wire_format.rs re-generates and checks, so drift on either side
// fails a test.
//
// Run from UI/: `bun test`.
import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { IncomingCommandSchema, SYSTEM_COMMAND_ID } from "../src/lib/protocol/command";
import { ProtocolMessageSchema } from "../src/lib/protocol/message";
import { PROTOCOL_VERSION, protocolVersionOf, SystemInfoSchema } from "../src/lib/protocol/system";

const fixturePath = fileURLToPath(
  new URL("../../protocol/tests/golden/wire_format.txt", import.meta.url),
);
const fixtures = readFileSync(fixturePath, "utf8")
  .split(/\r?\n/)
  .filter((line) => line.trim() !== "")
  .map((line) => {
    const [name, json] = line.split("\t");
    return { name, value: JSON.parse(json) };
  });

const commands = fixtures.filter((f) => f.name.startsWith("cmd."));
const messages = fixtures.filter((f) => !f.name.startsWith("cmd."));

describe("golden wire fixture", () => {
  test("covers every command and event family", () => {
    const families = (values: object[]) => [...new Set(values.map((v) => Object.keys(v)[0]))].sort();
    expect(families(commands.map((f) => f.value.command))).toEqual(
      ["Led", "Lidar", "Rangefinder", "Rfid", "Servo", "StepperMotor", "System"],
    );
    const events = messages.filter((f) => "ModuleEvent" in f.value).map((f) => f.value.ModuleEvent.event);
    expect(families(events)).toEqual(
      ["Button", "Imu", "Led", "Lidar", "Rangefinder", "RemoteReceiver", "Rfid", "Servo", "StepperMotor", "SysLog"],
    );
  });

  test.each(commands.map((f) => [f.name, f.value]))("%s parses as an IncomingCommand", (_name, value) => {
    const result = IncomingCommandSchema.safeParse(value);
    expect(result.error?.issues ?? []).toEqual([]);
  });

  test.each(messages.map((f) => [f.name, f.value]))("%s parses as a ProtocolMessage", (_name, value) => {
    const result = ProtocolMessageSchema.safeParse(value);
    expect(result.error?.issues ?? []).toEqual([]);
  });
});

describe("protocol v1", () => {
  test("the frontend version matches the firmware's", () => {
    const system = fixtures.find((f) => f.name === "msg.system")!.value.System;
    expect(system.protocol_version).toBe(PROTOCOL_VERSION);
  });

  test("the Rediscover command is the one the frontend sends", () => {
    const rediscover = fixtures.find((f) => f.name === "cmd.system.rediscover")!.value;
    expect(rediscover).toEqual({ id: SYSTEM_COMMAND_ID, command: { System: { Rediscover: {} } } });
  });

  test("firmware older than v1 reports no version and is treated as 0", () => {
    const system = { ...fixtures.find((f) => f.name === "msg.system")!.value.System };
    delete system.protocol_version;
    const parsed = SystemInfoSchema.parse(system);
    expect(protocolVersionOf(parsed)).toBe(0);
  });
});

describe("rejection", () => {
  test("unknown command families and events are rejected", () => {
    expect(IncomingCommandSchema.safeParse({ id: "u", command: { Toaster: {} } }).success).toBe(false);
    expect(
      ProtocolMessageSchema.safeParse({ ModuleEvent: { id: "u", event: { Toaster: {} } } }).success,
    ).toBe(false);
  });

  test("out-of-range values are rejected", () => {
    const step = { id: "u", command: { Lidar: { SetStep: { step: 0 } } } };
    expect(IncomingCommandSchema.safeParse(step).success).toBe(false);
  });
});
