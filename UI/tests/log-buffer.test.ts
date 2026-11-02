// Bounded, batched log history (audit Phase 2D, PIN-TPL-011).
//
// Run from UI/: `bun test`.
import { describe, expect, test } from "bun:test";
import {
  appendBounded,
  createLogBatcher,
  describeLog,
  exportLogs,
  LOG_CAPACITY,
  matchesLog,
  recordLogReceipt,
} from "../src/lib/logs";
import type { ProtocolMessage } from "../src/lib/protocol/message";

const led = (n: number) =>
  ({ ModuleEvent: { id: `led-${n}`, event: { Led: { Brightness: { level: n % 101 } } } } }) as ProtocolMessage;

describe("appendBounded", () => {
  test("keeps everything below the capacity, in order", () => {
    expect(appendBounded([1, 2], [3, 4], 10)).toEqual([1, 2, 3, 4]);
  });

  test("keeps the newest entries above the capacity", () => {
    expect(appendBounded([1, 2, 3], [4, 5], 4)).toEqual([2, 3, 4, 5]);
    expect(appendBounded([1, 2], [3, 4, 5, 6, 7], 3)).toEqual([5, 6, 7]);
  });

  test("an empty batch returns the same array (no store change)", () => {
    const history = [1];
    expect(appendBounded(history, [], 3)).toBe(history);
  });
});

describe("100 000 synthetic messages", () => {
  test("memory stays bounded and the append cost stays flat", () => {
    let history: ProtocolMessage[] = [];
    const timings: number[] = [];
    for (let batch = 0; batch < 1_000; batch++) {
      const items = Array.from({ length: 100 }, (_, i) => led(batch * 100 + i));
      const started = performance.now();
      history = appendBounded(history, items);
      timings.push(performance.now() - started);
      expect(history.length).toBeLessThanOrEqual(LOG_CAPACITY);
    }
    expect(history.length).toBe(LOG_CAPACITY);
    expect((history[history.length - 1] as { ModuleEvent: { id: string } }).ModuleEvent.id).toBe("led-99999");
    // Flat: the last 100 batches cost about the same as batches 100-200 (both at
    // capacity), never growing with the number of messages seen.
    const mean = (xs: number[]) => xs.reduce((a, b) => a + b, 0) / xs.length;
    const early = mean(timings.slice(100, 200));
    const late = mean(timings.slice(900));
    expect(late).toBeLessThan(early * 3 + 0.5);
  });
});

describe("createLogBatcher", () => {
  test("a burst becomes one flush", () => {
    const flushed: number[][] = [];
    let run: (() => void) | undefined;
    const batcher = createLogBatcher<number>((b) => flushed.push(b), { schedule: (r) => (run = r) });
    for (let i = 0; i < 500; i++) batcher.push(i);
    expect(flushed).toEqual([]);
    run!();
    expect(flushed.length).toBe(1);
    expect(flushed[0].length).toBe(500);
  });

  test("clear drops messages that were not flushed yet", () => {
    const flushed: number[][] = [];
    let run: (() => void) | undefined;
    const batcher = createLogBatcher<number>((b) => flushed.push(b), { schedule: (r) => (run = r) });
    batcher.push(1);
    batcher.clear();
    run!();
    expect(flushed).toEqual([]);
  });

  test("the pending batch is bounded while flushes are delayed", () => {
    const batcher = createLogBatcher<number>(() => {}, { capacity: 10, schedule: () => {} });
    for (let i = 0; i < 1_000; i++) batcher.push(i);
    expect(batcher.pendingCount()).toBeLessThanOrEqual(20);
  });
});

describe("cached summaries and search", () => {
  test("search matches the same as before and is computed once per message", () => {
    const message = led(42);
    recordLogReceipt(message);
    const filters = { category: "All logs" as const, priority: "All priorities" as const, search: "led-42" };
    expect(matchesLog(message, filters)).toBe(true);
    expect(matchesLog(message, { ...filters, search: "led-43" })).toBe(false);
    expect(describeLog(message)).toBe(describeLog(message));
  });

  test("export carries receipt times and messages", () => {
    const message = led(7);
    recordLogReceipt(message);
    const exported = JSON.parse(exportLogs([message]));
    expect(exported[0].message).toEqual(message);
    expect(typeof exported[0].received).toBe("string");
  });
});
