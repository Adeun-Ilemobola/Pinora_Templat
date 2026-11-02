import type { ProtocolMessage } from "./protocol/message";
import type { LogPriority } from "./protocol/event";

// Receipt times belong to the frontend, not the firmware's wire payload.
const receivedAt = new WeakMap<ProtocolMessage, number>();

export function recordLogReceipt(message: ProtocolMessage) {
  receivedAt.set(message, Date.now());
}

export type LogCategory = "All logs" | "Registration events" | "System logs" | "System telemetry" | "Module events";

export type LogFilters = {
  category: LogCategory;
  priority: LogPriority | "All priorities";
  search: string;
};

export const logCategories: LogCategory[] = ["All logs", "Registration events", "System logs", "System telemetry", "Module events"];
export const logPriorities: LogFilters["priority"][] = ["All priorities", "Low", "Medium", "High", "Critical"];

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

// Returned for shapes the helpers do not recognize. Tauri validates every line before
// it reaches the frontend (Phase 1C), so this is defense in depth: before, such input
// threw inside LogViewer's render and crashed the view (PIN-TPL-006).
function unrecognized(time: number | undefined) {
  return {
    category: "Module events" as const,
    variant: "Unrecognized",
    source: "Unknown",
    text: "Unrecognized message",
    time,
    priority: undefined
  };
}

/** Most entries kept in the log history (audit Phase 2D, PIN-TPL-011). Older entries
 *  are dropped; use Export to keep a session. */
export const LOG_CAPACITY = 5_000;

/** Appends a batch to the history, keeping the newest `capacity` entries. The history
 *  is copied once per batch, not once per message (before 2D every message copied
 *  the whole, unbounded array). */
export function appendBounded<T>(history: readonly T[], batch: readonly T[], capacity = LOG_CAPACITY): T[] {
  if (!batch.length) return history as T[];
  if (history.length + batch.length <= capacity) return history.concat(batch);
  const fromBatch = Math.min(batch.length, capacity);
  const fromHistory = capacity - fromBatch;
  return history.slice(history.length - fromHistory).concat(batch.slice(batch.length - fromBatch));
}

/** Collects messages and flushes them together, at most once per `delayMs`, so a
 *  burst of telemetry causes one store update instead of one per message. The pending
 *  batch is itself bounded by `capacity`. */
export function createLogBatcher<T>(
  flush: (batch: T[]) => void,
  options: { delayMs?: number; capacity?: number; schedule?: (run: () => void, ms: number) => void } = {},
) {
  const { delayMs = 50, capacity = LOG_CAPACITY, schedule = (run, ms) => void setTimeout(run, ms) } = options;
  let pending: T[] = [];
  let scheduled = false;
  const run = () => {
    scheduled = false;
    if (!pending.length) return;
    const batch = pending;
    pending = [];
    flush(batch);
  };
  return {
    push(item: T) {
      pending.push(item);
      if (pending.length > 2 * capacity) pending = pending.slice(-capacity);
      if (!scheduled) {
        scheduled = true;
        schedule(run, delayMs);
      }
    },
    /** Drops messages not yet flushed (on clear or reconnect). */
    clear() {
      pending = [];
    },
    pendingCount: () => pending.length,
  };
}

/** The history as JSON for saving: each entry with its receipt time. */
export function exportLogs(messages: readonly ProtocolMessage[]): string {
  return JSON.stringify(
    messages.map((message) => {
      const time = receivedAt.get(message);
      return { received: time === undefined ? null : new Date(time).toISOString(), message };
    }),
    null,
    2,
  );
}

type LogEntry = ReturnType<typeof describeUncached>;
// Summaries and search text are computed once per message (before 2D, every filter
// change and every new message re-described and re-stringified the whole history).
const described = new WeakMap<object, LogEntry>();
const searchText = new WeakMap<object, string>();

/** Summarizes a log entry for display. Never throws, whatever the input shape. */
export function describeLog(message: ProtocolMessage): LogEntry {
  if (!isRecord(message)) return describeUncached(message);
  const cached = described.get(message);
  if (cached && cached.time === receivedAt.get(message)) return cached;
  const entry = describeUncached(message);
  described.set(message, entry);
  return entry;
}

function describeUncached(message: ProtocolMessage) {
  const time = isRecord(message) ? receivedAt.get(message) : undefined;

  if (!isRecord(message)) {
    return unrecognized(time);
  }

  if ("Registration" in message && isRecord(message.Registration)) {
    const registration = message.Registration;

    return {
      category: "Registration events",
      variant: registration.module_type,
      source: registration.id,
      text: `Registered ${registration.module_type} · ${registration.lool_up_id || registration.id}`,
      time,
      priority: undefined
    };
  }

  if ("System" in message && isRecord(message.System)) {
    return {
      category: "System telemetry",
      variant: "System",
      source: "Device",
      text: `System snapshot · ESP-IDF ${message.System.esp_idf_version} · Free heap ${message.System.current_free_heap}`,
      time,
      priority: undefined
    };
  }

  const modulePackage = "ModuleEvent" in message ? message.ModuleEvent : undefined;
  if (!isRecord(modulePackage) || !isRecord(modulePackage.event)) {
    return unrecognized(time);
  }
  const { id, event } = modulePackage;

  if ("SysLog" in event && isRecord(event.SysLog)) {
    return {
      category: "System logs",
      variant: "SysLog",
      source: id,
      text: event.SysLog.text,
      time,
      priority: event.SysLog.priority
    };
  }

  const first = Object.entries(event)[0];
  if (!first) {
    return unrecognized(time);
  }
  const [kind, payload] = first;
  const variant = typeof payload === "object" && payload !== null ? Object.keys(payload).join(", ") : String(payload);

  return {
    category: "Module events",
    variant: kind,
    source: id,
    text: `${kind} · ${variant}`,
    time,
    priority: undefined
  };
}

export function matchesLog(message: ProtocolMessage, filters: LogFilters): boolean {
  const entry = describeLog(message);

  if (filters.category !== "All logs" && entry.category !== filters.category)
    return false;

  if (filters.priority !== "All priorities" && entry.priority !== filters.priority)
    return false;

  const query = filters.search.trim().toLowerCase();
  if (!query) return true;
  let text = isRecord(message) ? searchText.get(message) : undefined;
  if (text === undefined) {
    let json: string;
    try {
      json = JSON.stringify(message) ?? "";
    } catch {
      json = "";
    }
    text = `${entry.category} ${entry.text} ${json}`.toLowerCase();
    if (isRecord(message)) searchText.set(message, text);
  }
  return text.includes(query);
}
