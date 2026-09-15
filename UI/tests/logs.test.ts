import { expect, test } from "bun:test";
import { describeLog, matchesLog, recordLogReceipt, logPriorities, type LogFilters } from "../src/lib/logs";
import type { ProtocolMessage } from "../src/lib/protocol/message";

const registration: ProtocolMessage = {
  Registration: {
    id: "servo-1",
    module_type: "Servo",
    lool_up_id: "servo_x",
    parent_id: "lidar"
  }
};

const systemLogs: ProtocolMessage[] = logPriorities.slice(1).map(priority => ({
  ModuleEvent: {
    id: "sensor",

    event: {
      SysLog: {
        priority: priority as "Low",
        text: "Range failure",
        raw_err: "Timeout details"
      }
    }
  }
}));

const telemetry: ProtocolMessage = {
  System: {
    esp_idf_version: "5",
    total_heap: "100",
    current_free_heap: "50",
    lowest_free_heap: "40",
    largest_allocation: "20",
    maximum_app_slot: "1000",
    flash: "2000"
  }
};

const event: ProtocolMessage = {
  ModuleEvent: {
    id: "servo-1",

    event: {
      Servo: {
        Angle: 10
      }
    }
  }
};

const logs = [registration, ...systemLogs, telemetry, event];

const defaults: LogFilters = {
  category: "All logs",
  priority: "All priorities",
  search: ""
};

test("real protocol categories and all priorities filter without modifying history", () => {
  const before = JSON.stringify(logs);
  expect(logs.filter(m => matchesLog(m, defaults))).toHaveLength(7);

  for (const [category, count] of [
    ["Registration events", 1],
    ["System logs", 4],
    ["System telemetry", 1],
    ["Module events", 1]
  ] as const) {
    expect(logs.filter(m => matchesLog(m, {
      ...defaults,
      category
    }))).toHaveLength(count);
  }

  for (const priority of logPriorities.slice(1)) {
    expect(logs.filter(m => matchesLog(m, {
      ...defaults,
      priority
    }))).toHaveLength(1);
  }

  expect(logs.filter(m => matchesLog(m, {
    ...defaults,
    search: " TIMEOUT "
  }))).toHaveLength(4);

  expect(logs.filter(m => matchesLog(m, {
    ...defaults,
    search: "servo_x"
  }))).toEqual([registration]);

  expect(logs.filter(m => matchesLog(m, {
    ...defaults,
    category: "Registration events",
    search: "failure"
  }))).toHaveLength(0);

  expect(JSON.stringify(logs)).toBe(before);
});

test(
  "receipt metadata leaves registration payload unchanged and survives page remounts",
  () => {
    const before = JSON.stringify(registration);
    recordLogReceipt(registration);
    expect(describeLog(registration).time).toBeGreaterThan(0);
    expect(describeLog(registration).source).toBe("servo-1");
    expect(describeLog(registration).text).toContain("servo_x");
    expect(JSON.stringify(registration)).toBe(before);
  }
);
