import { memo, useRef } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Terminal, Trash2 } from "lucide-react";
import { useModuleFront } from "@/lib/Modulefront";
import type { ProtocolMessage } from "@/lib/protocol/message";
import type { LogPriority } from "@/lib/protocol/event";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardAction,
  CardContent,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
const priorities: Record<LogPriority, string> = {
  Low: "text-muted-foreground",
  Medium: "bg-amber-500/10 text-amber-800 dark:text-amber-300",
  High: "bg-orange-500/10 text-orange-800 dark:text-orange-300",
  Critical: "bg-destructive/10 text-destructive font-semibold",
};
const LogRow = memo(function LogRow({
  message,
  index,
}: {
  message: ProtocolMessage;
  index: number;
}) {
  const category =
    "Registration" in message
      ? "Registration"
      : "System" in message
        ? "System"
        : "ModuleEvent";
  const packet = "ModuleEvent" in message ? message.ModuleEvent : null;
  const syslog =
    packet && "SysLog" in packet.event ? packet.event.SysLog : null;
  const payload =
    "Registration" in message
      ? message.Registration
      : "System" in message
        ? message.System
        : message.ModuleEvent.event;
  return (
    <div className="flex min-w-0 flex-wrap items-start gap-2 border-b px-4 py-3 text-xs">
      <span className="w-10 shrink-0 pt-1 text-muted-foreground">
        {index + 1}
      </span>
      <Badge variant="outline">{category}</Badge>
      {syslog && (
        <Badge variant="outline" className={priorities[syslog.priority]}>
          {syslog.priority}
        </Badge>
      )}
      <div className="min-w-0 basis-full space-y-1 font-mono sm:basis-0 sm:flex-1">
        {packet && (
          <div className="break-all text-muted-foreground">
            {packet.id}
            {syslog ? " · SysLog" : ""}
          </div>
        )}
        <p className="whitespace-pre-wrap break-all">
          {syslog ? syslog.text : JSON.stringify(payload)}
        </p>
        {syslog?.raw_err && (
          <p className="whitespace-pre-wrap break-all text-destructive">
            {syslog.raw_err}
          </p>
        )}
      </div>
    </div>
  );
});

/** Only visible traffic is formatted and mounted; measured heights support wrapped payloads. */
export function LogViewer() {
  const logs = useModuleFront((s) => s.logs);
  const clear = useModuleFront((s) => s.clearLogs);
  const scrollRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: logs.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => 76,
    overscan: 6,
  });
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Terminal className="size-4" />
          Protocol logs{" "}
          <Badge variant="secondary">{logs.length.toLocaleString()}</Badge>
        </CardTitle>
        <CardDescription>Incoming device traffic</CardDescription>
        <CardAction>
          <Button
            variant="ghost"
            size="sm"
            disabled={!logs.length}
            onClick={() => {
              clear();
              scrollRef.current?.scrollTo(0, 0);
            }}
          >
            <Trash2 />
            Clear
          </Button>
        </CardAction>
      </CardHeader>
      <CardContent>
        <div
          ref={scrollRef}
          tabIndex={0}
          aria-label="Protocol log entries"
          className="relative h-80 overflow-auto rounded-xl border bg-background focus-visible:outline-2 focus-visible:outline-ring"
        >
          {!logs.length ? (
            <div className="flex h-full items-center justify-center p-6 text-center text-sm text-muted-foreground">
              No messages yet. Incoming protocol traffic will appear here.
            </div>
          ) : (
            <div
              className="relative w-full"
              style={{ height: virtualizer.getTotalSize() }}
            >
              {virtualizer.getVirtualItems().map((row) => (
                <div
                  key={row.key}
                  data-index={row.index}
                  ref={virtualizer.measureElement}
                  className="absolute left-0 top-0 w-full"
                  style={{ transform: `translateY(${row.start}px)` }}
                >
                  <LogRow message={logs[row.index]} index={row.index} />
                </div>
              ))}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
