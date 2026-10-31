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

/** Summarizes a log entry for display. Never throws, whatever the input shape. */
export function describeLog(message: ProtocolMessage) {
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
  return !query || `${entry.category} ${entry.text} ${JSON.stringify(message)}`.toLowerCase().includes(query);
}
