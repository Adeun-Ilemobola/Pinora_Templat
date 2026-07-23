import { useEffect, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import {
  ActivityLogEntry,
  InComingMessage,
  useListenStore,
} from "@/lib/ListenStore";

export default function LogFrame() {
  const logs = useListenStore((state) => state.activityLogs);
  const clearLogs = useListenStore((state) => state.clearActivityLogs);

  return <LogFramePage logs={logs} onClear={clearLogs} />;
}

function LogFramePage({
  logs,
  onClear,
}: {
  logs: ActivityLogEntry[];
  onClear: () => void;
}) {
  const parentRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: logs.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 82,
    overscan: 8,
  });

  useEffect(() => {
    if (logs.length > 0) {
      virtualizer.scrollToIndex(logs.length - 1, { align: "end" });
    }
  }, [logs.length, virtualizer]);

  return (
    <main className="flex h-full min-h-0 w-full flex-col p-4">
      <div className="mb-3 flex items-center justify-between gap-4">
        <div>
          <h1 className="text-lg font-semibold">Activity log</h1>
          <p className="text-sm text-muted-foreground">
            Registrations and module events received from the device.
          </p>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={onClear}
          disabled={logs.length === 0}
        >
          Clear
        </Button>
      </div>

      <div
        ref={parentRef}
        className="min-h-0 flex-1 overflow-auto rounded-lg border"
      >
        {logs.length === 0 ? (
          <div className="flex h-full min-h-48 items-center justify-center px-6 text-center text-sm text-muted-foreground">
            Device activity will appear here after you connect.
          </div>
        ) : (
          <div
            style={{
              height: `${virtualizer.getTotalSize()}px`,
              width: "100%",
              position: "relative",
            }}
          >
            {virtualizer.getVirtualItems().map((virtualItem) => {
              const log = logs[virtualItem.index];

              return (
                <div
                  key={log.id}
                  ref={virtualizer.measureElement}
                  data-index={virtualItem.index}
                  style={{
                    position: "absolute",
                    top: 0,
                    left: 0,
                    width: "100%",
                    transform: `translateY(${virtualItem.start}px)`,
                  }}
                >
                  <LogCard log={log} />
                  <Separator />
                </div>
              );
            })}
          </div>
        )}
      </div>
    </main>
  );
}

function LogCard({ log }: { log: ActivityLogEntry }) {
  const { label, summary, details } = describeMessage(log.message);
  const isRegistration = log.message.type === "Registration";

  return (
    <div className="flex gap-3 px-4 py-3">
      <time className="w-24 shrink-0 pt-0.5 font-mono text-xs text-muted-foreground">
        {new Date(log.receivedAt).toLocaleTimeString()}
      </time>
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <span
            className={`shrink-0 rounded px-2 py-0.5 text-xs font-semibold uppercase ${
              isRegistration
                ? "bg-green-500/15 text-green-600 dark:text-green-400"
                : "bg-blue-500/15 text-blue-600 dark:text-blue-400"
            }`}
          >
            {label}
          </span>
          <span className="text-sm font-medium">{summary}</span>
        </div>
        <p className="mt-1 break-words font-mono text-xs text-muted-foreground">
          {details}
        </p>
      </div>
    </div>
  );
}

function describeMessage(message: InComingMessage) {
  if (message.type === "Registration") {
    const registration = message.payload;
    return {
      label: "Registration",
      summary: `${registration.module_type} module registered`,
      details: `id=${registration.id} · lookup=${registration.lool_up_id} · parent=${registration.parent_id}`,
    };
  }

  const moduleEvent = message.payload;
  const event = moduleEvent.event;
  const eventData = Object.entries(event)
    .filter(([key]) => key !== "event_type" && key !== "id")
    .map(([key, value]) => `${key}=${String(value)}`)
    .join(" · ");

  return {
    label: "Event",
    summary: `${moduleEvent.module_type} ${"event_type" in event ? event.event_type : "Log"}`,
    details: ["id" in event ? `id=${event.id}` : "", eventData].filter(Boolean).join(" · "),
  };
}
