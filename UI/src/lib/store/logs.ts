// The protocol log history: bounded and batched (audit Phase 2D, PIN-TPL-011).
import { appendBounded, createLogBatcher } from "../logs";
import type { ProtocolMessage } from "../protocol/message";
import type { LogSlice, Slice } from "./types";

export const createLogSlice: Slice<LogSlice> = (set) => {
  // One store update per 50 ms burst of messages, bounded to LOG_CAPACITY entries.
  const batcher = createLogBatcher<ProtocolMessage>((batch) =>
    set((state) => ({ logs: appendBounded(state.logs, batch) })),
  );
  return {
    logs: [],
    pushLog: (message) => batcher.push(message),
    clearLogs: () => {
      batcher.clear();
      set({ logs: [] });
    },
  };
};
