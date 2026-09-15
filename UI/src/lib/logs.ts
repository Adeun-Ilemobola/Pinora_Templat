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

export function describeLog(message: ProtocolMessage) {
  const time = receivedAt.get(message);

  if ("Registration" in message) {
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

  if ("System" in message) {
    return {
      category: "System telemetry",
      variant: "System",
      source: "Device",
      text: `System snapshot · ESP-IDF ${message.System.esp_idf_version} · Free heap ${message.System.current_free_heap}`,
      time,
      priority: undefined
    };
  }

  const {
    id,
    event
  } = message.ModuleEvent;

  if ("SysLog" in event) {
    return {
      category: "System logs",
      variant: "SysLog",
      source: id,
      text: event.SysLog.text,
      time,
      priority: event.SysLog.priority
    };
  }

  const [kind, payload] = Object.entries(event)[0];
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
