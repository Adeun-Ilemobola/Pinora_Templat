// Characterization tests for the log helpers (audit Phase 0, PIN-TPL-008).
//
// These record CURRENT behavior so structural refactors can prove nothing changed.
// Cases marked "current behavior, see PIN-TPL-006" document a known defect: the
// helpers throw on JSON that is not a recognized ProtocolMessage. When PIN-TPL-006 is
// fixed, those expectations change deliberately.
//
// Run from UI/: `bun test`. Lives outside src/ so `tsc` (which has no bun:test types)
// does not type-check it.
import { describe, expect, test } from "bun:test";
import { describeLog, matchesLog, recordLogReceipt, logCategories, logPriorities } from "../src/lib/logs";
import type { LogFilters } from "../src/lib/logs";

const all: LogFilters = { category: "All logs", priority: "All priorities", search: "" };

const registration = { Registration: { id: "uuid-1", module_type: "Led", lool_up_id: "led1", parent_id: "" } } as const;
const system = {
  System: {
    esp_idf_version: "v5.5.3",
    total_heap: "300 KiB",
    current_free_heap: "200 KiB",
    lowest_free_heap: "190 KiB",
    largest_allocation: "100 KiB",
    maximum_app_slot: "3 MiB",
    flash: "4 MiB",
  },
} as const;
const syslog = {
  ModuleEvent: { id: "log", event: { SysLog: { text: "Rangefinder error", raw_err: null, priority: "High" } } },
} as const;
const led = { ModuleEvent: { id: "uuid-1", event: { Led: { Brightness: { level: 80 } } } } } as const;

describe("describeLog", () => {
  test("registration", () => {
    expect(describeLog(registration as any)).toEqual({
      category: "Registration events",
      variant: "Led",
      source: "uuid-1",
      text: "Registered Led · led1",
      time: undefined,
      priority: undefined,
    });
  });

  test("registration without lookup name falls back to id", () => {
    const msg = { Registration: { ...registration.Registration, lool_up_id: "" } };
    expect(describeLog(msg as any).text).toBe("Registered Led · uuid-1");
  });

  test("system", () => {
    expect(describeLog(system as any)).toEqual({
      category: "System telemetry",
      variant: "System",
      source: "Device",
      text: "System snapshot · ESP-IDF v5.5.3 · Free heap 200 KiB",
      time: undefined,
      priority: undefined,
    });
  });

  test("syslog", () => {
    expect(describeLog(syslog as any)).toEqual({
      category: "System logs",
      variant: "SysLog",
      source: "log",
      text: "Rangefinder error",
      time: undefined,
      priority: "High",
    });
  });

  test("module event", () => {
    expect(describeLog(led as any)).toEqual({
      category: "Module events",
      variant: "Led",
      source: "uuid-1",
      text: "Led · Brightness",
      time: undefined,
      priority: undefined,
    });
  });

  test("bare-string payload variant", () => {
    const msg = { ModuleEvent: { id: "x", event: { Rangefinder: "StartRanging" } } };
    expect(describeLog(msg as any).text).toBe("Rangefinder · StartRanging");
  });

  test("receipt time is attached by recordLogReceipt", () => {
    const msg = { ModuleEvent: { id: "uuid-1", event: { Led: { Brightness: { level: 1 } } } } };
    recordLogReceipt(msg as any);
    expect(typeof describeLog(msg as any).time).toBe("number");
  });

  // current behavior, see PIN-TPL-006
  test("throws on unknown top-level object", () => {
    expect(() => describeLog({ Foo: 1 } as any)).toThrow(TypeError);
  });
  test("throws on empty object", () => {
    expect(() => describeLog({} as any)).toThrow(TypeError);
  });
  test("throws on ModuleEvent with empty event", () => {
    expect(() => describeLog({ ModuleEvent: { id: "x", event: {} } } as any)).toThrow(TypeError);
  });
});

describe("matchesLog", () => {
  const samples = [registration, system, syslog, led] as any[];

  test("All logs matches everything", () => {
    expect(samples.map((m) => matchesLog(m, all))).toEqual([true, true, true, true]);
  });

  test("category filter", () => {
    const pick = (category: LogFilters["category"]) => samples.map((m) => matchesLog(m, { ...all, category }));
    expect(pick("Registration events")).toEqual([true, false, false, false]);
    expect(pick("System telemetry")).toEqual([false, true, false, false]);
    expect(pick("System logs")).toEqual([false, false, true, false]);
    expect(pick("Module events")).toEqual([false, false, false, true]);
  });

  test("priority filter only matches syslog entries with that priority", () => {
    expect(samples.map((m) => matchesLog(m, { ...all, priority: "High" }))).toEqual([false, false, true, false]);
    expect(samples.map((m) => matchesLog(m, { ...all, priority: "Low" }))).toEqual([false, false, false, false]);
  });

  test("search is case-insensitive over category, text and raw JSON", () => {
    expect(matchesLog(led as any, { ...all, search: "BRIGHTNESS" })).toBe(true);
    expect(matchesLog(led as any, { ...all, search: "level\":80" })).toBe(true);
    expect(matchesLog(led as any, { ...all, search: "module events" })).toBe(true);
    expect(matchesLog(led as any, { ...all, search: "servo" })).toBe(false);
    expect(matchesLog(led as any, { ...all, search: "   " })).toBe(true);
  });

  // current behavior, see PIN-TPL-006
  test("throws on unrecognized JSON", () => {
    expect(() => matchesLog({ Foo: 1 } as any, all)).toThrow(TypeError);
  });
});

test("filter option lists", () => {
  expect(logCategories).toEqual(["All logs", "Registration events", "System logs", "System telemetry", "Module events"]);
  expect(logPriorities).toEqual(["All priorities", "Low", "Medium", "High", "Critical"]);
});
