import { memo, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Collapsible } from "@base-ui/react/collapsible";
import { useModuleFront } from "@/lib/Modulefront";
import type { ProtocolMessage } from "@/lib/protocol/message";
import type { LogPriority } from "@/lib/protocol/event";
import {
  describeLog,
  matchesLog,
  logCategories,
  logPriorities,
  type LogFilters,
} from "@/lib/logs";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@/components/ui/select";

const priorities: Record<LogPriority, string> = {
  Low: "text-muted-foreground",
  Medium: "bg-amber-500/10 text-amber-800 dark:text-amber-300",
  High: "bg-orange-500/10 text-orange-800 dark:text-orange-300",
  Critical: "bg-destructive/10 text-destructive font-semibold",
};

const defaults: LogFilters = {
  category: "All logs",
  priority: "All priorities",
  search: "",
};

const LogRow = memo(function LogRow({
  message,
  index,
}: {
  message: ProtocolMessage;
  index: number;
}) {
  const entry = describeLog(message);
  const [expanded, setExpanded] = useState(false);
  // @ts-expect-error TS7053 -- temporary
  const priorityClass = entry.priority ? priorities[entry.priority] : undefined;

  return (
    <Collapsible.Root
      open={expanded}
      onOpenChange={setExpanded}
      className="border-b px-4 py-3 text-xs"
    >
      <div className="flex flex-wrap items-center gap-2">
        <span className="font-mono text-muted-foreground">#{index + 1}</span>
        <time
          dateTime={entry.time ? new Date(entry.time).toISOString() : undefined}
          title={
            entry.time
              ? new Date(entry.time).toLocaleString()
              : "Receipt time unavailable"
          }
          className="font-mono text-muted-foreground"
        >
          {entry.time
            ? new Date(entry.time).toLocaleTimeString()
            : "Time unavailable"}
        </time>
        <Badge
          variant={
            entry.category === "Registration events" ? "secondary" : "outline"
          }
        >
          {entry.category}
        </Badge>
        <Badge variant="outline">{entry.variant}</Badge>
        {entry.priority && (
          <Badge variant="outline" className={priorityClass}>
            {entry.priority}
          </Badge>
        )}
        <span className="min-w-0 break-all font-mono text-muted-foreground">
          {entry.source}
        </span>
      </div>
      <p className="mt-2 line-clamp-2 whitespace-pre-wrap break-all text-sm">
        {entry.text}
      </p>
      <Collapsible.Trigger
        render={<Button variant="ghost" size="sm" className="mt-1" />}
      >
        {expanded ? "Hide details" : "Show details"}
      </Collapsible.Trigger>
      <Collapsible.Panel>
        {expanded && (
          <pre className="mt-2 max-h-80 overflow-auto whitespace-pre-wrap break-all rounded-md bg-muted/40 p-3 font-mono text-xs">
            {JSON.stringify(message, null, 2)}
          </pre>
        )}
      </Collapsible.Panel>
    </Collapsible.Root>
  );
});

export function LogViewer() {
  const logs = useModuleFront((s) => s.logs);
  const clear = useModuleFront((s) => s.clearLogs);
  const [filters, setFilters] = useState<LogFilters>(defaults);
  const scrollRef = useRef<HTMLDivElement>(null);

  // Keep original positions for stable row identity; filtering never edits history.
  const visible = useMemo(
    () =>
      logs.flatMap((message, index) =>
        matchesLog(message, filters)
          ? [
              {
                message,
                index,
              },
            ]
          : [],
      ),
    [logs, filters],
  );

  const active =
    filters.category !== defaults.category ||
    filters.priority !== defaults.priority ||
    !!filters.search.trim();

  const virtualizer = useVirtualizer({
    count: visible.length,
    getScrollElement: () => scrollRef.current,
    getItemKey: (index) => visible[index].index,
    estimateSize: () => 126,
    overscan: 6,
  });

  const update = (next: LogFilters) => {
    setFilters(next);
    scrollRef.current?.scrollTo(0, 0);
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Incoming protocol traffic</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex flex-wrap items-end gap-3">
          <div className="space-y-1">
            <label
              id="log-category-label"
              className="text-xs text-muted-foreground"
            >
              Event type
            </label>
            <Select
              value={filters.category}
              onValueChange={(category) =>
                category &&
                update({
                  ...filters,
                  category,
                  priority: "All priorities",
                })
              }
            >
              <SelectTrigger aria-labelledby="log-category-label">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {logCategories.map((value) => (
                  <SelectItem key={value} value={value}>
                    {value}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-1">
            <label
              id="log-priority-label"
              className="text-xs text-muted-foreground"
            >
              SysLog priority
            </label>
            <Select
              value={filters.priority}
              disabled={!["All logs", "System logs"].includes(filters.category)}
              onValueChange={(priority) =>
                priority &&
                update({
                  ...filters,
                  priority,
                })
              }
            >
              <SelectTrigger aria-labelledby="log-priority-label">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {logPriorities.map((value) => (
                  <SelectItem key={value} value={value}>
                    {value}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="min-w-48 flex-1 space-y-1">
            <label
              htmlFor="log-search"
              className="text-xs text-muted-foreground"
            >
              Search messages, sources, and details
            </label>
            <Input
              id="log-search"
              type="search"
              value={filters.search}
              onChange={(event) =>
                update({
                  ...filters,
                  search: event.target.value,
                })
              }
              placeholder="Search logs…"
            />
          </div>
          <Button
            variant="outline"
            disabled={!active}
            onClick={() => update(defaults)}
          >
            Reset filters
          </Button>
        </div>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <p role="status" className="text-xs text-muted-foreground">
            {`${visible.length.toLocaleString()} of ${logs.length.toLocaleString()} entries `}
            {active && <Badge variant="secondary">Filters active</Badge>}
          </p>
          <Button
            variant="ghost"
            size="sm"
            disabled={!logs.length}
            onClick={() => {
              clear();
              scrollRef.current?.scrollTo(0, 0);
            }}
          >
            Clear history
          </Button>
        </div>
        <div
          ref={scrollRef}
          tabIndex={0}
          aria-label="Protocol log entries"
          className="relative h-[60vh] min-h-80 overflow-auto rounded-xl border bg-background focus-visible:outline-2 focus-visible:outline-ring"
        >
          {!visible.length ? (
            <div className="flex h-full items-center justify-center p-6 text-center text-sm text-muted-foreground">
              {logs.length
                ? "No entries match these filters. Reset filters to view all logs."
                : "No messages yet. Incoming protocol traffic will appear here."}
            </div>
          ) : (
            <div
              className="relative w-full"
              style={{
                height: virtualizer.getTotalSize(),
              }}
            >
              {virtualizer.getVirtualItems().map((row) => (
                <div
                  key={row.key}
                  data-index={row.index}
                  ref={virtualizer.measureElement}
                  className="absolute left-0 top-0 w-full"
                  style={{
                    transform: `translateY(${row.start}px)`,
                  }}
                >
                  <LogRow
                    message={visible[row.index].message}
                    index={visible[row.index].index}
                  />
                </div>
              ))}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
