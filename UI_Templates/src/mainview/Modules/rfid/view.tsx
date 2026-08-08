import { memo, useEffect, useState } from "react";
import {
  CheckCircle2,
  CreditCard,
  Radio,
  RefreshCw,
  Send,
  TriangleAlert,
} from "lucide-react";

import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import type { Commandtype as Command } from "@shared/Protocol/ModuleCommand";
import type { RfidModuleDefinition, RfidModeType } from "./definition";

type RfidCardProps = {
  module: RfidModuleDefinition;
  sendCommand: (command: Command) => Promise<void>;
  disabled?: boolean;
};

type WritableId = {
  value: string;
  bytes: number[];
};

const SCAN_DISPLAY_TIME_MS = 12_000;

function uuidToBytes(value: string): number[] {
  const hex = value.replace(/-/g, "");

  if (!/^[0-9a-f]{32}$/i.test(hex)) {
    throw new Error("The generated UUID is not valid.");
  }

  return Array.from({ length: 16 }, (_, index) =>
    Number.parseInt(hex.slice(index * 2, index * 2 + 2), 16),
  );
}

function generateWritableId(): WritableId {
  const value = crypto.randomUUID();
  return { value, bytes: uuidToBytes(value) };
}

export const RfidCard = memo(function RfidCard({
  module,
  sendCommand,
  disabled = false,
}: RfidCardProps) {
  const [currentScan, setCurrentScan] = useState(module.state.recentScan);
  const [writableId, setWritableId] = useState<WritableId>(generateWritableId);
  const [writeStatus, setWriteStatus] = useState(module.state.writeMsg);
  const [isSending, setIsSending] = useState(false);

  useEffect(() => {
    if (!module.state.recentScan) {
      setCurrentScan(null);
      return;
    }

    setCurrentScan(module.state.recentScan);
    const clearScan = window.setTimeout(() => {
      setCurrentScan(null);
    }, SCAN_DISPLAY_TIME_MS);

    return () => window.clearTimeout(clearScan);
  }, [module.state.recentScan]);

  useEffect(() => {
    setWriteStatus(module.state.writeMsg);
  }, [module.state.writeMsg]);

  function setMode(mode: RfidModeType) {
    if (mode === module.state.mode) return;

    void sendCommand({
      id: module.id,
      module_type: "Rfid",
      payload: { command: mode === "Read" ? "ReadMode" : "WriteMode" },
    });
  }

  function regenerateId() {
    setWritableId(generateWritableId());
    setWriteStatus(null);
  }

  async function sendId() {
    setWriteStatus(null);
    setIsSending(true);

    try {
      await sendCommand({
        id: module.id,
        module_type: "Rfid",
        payload: {
          command: "WritePayload",
          data: writableId.bytes,
        },
      });
    } finally {
      setIsSending(false);
    }
  }

  const isReadMode = module.state.mode === "Read";
  const writeMessage = writeStatus;

  return (
    <Card className="relative w-full border-0 bg-card shadow-sm ring-1 ring-foreground/10">
      <CardHeader className="border-b pb-5">
        <div className="flex min-w-0 items-start justify-between gap-6">
          <div className="flex min-w-0 items-center gap-3.5">
            <div className="grid size-11 shrink-0 place-items-center rounded-2xl bg-primary/10 text-primary ring-1 ring-primary/15">
              <Radio className="size-5" />
            </div>

            <div className="min-w-0">
              <div className="flex items-center gap-2">
                <h2 className="font-heading text-xl font-medium">RFID station</h2>
                <Badge variant="outline" className="hidden sm:inline-flex">
                  {module.lool_up_id}
                </Badge>
              </div>
              <p className="mt-0.5 truncate text-sm text-muted-foreground">
                Scan a card or write a generated identifier
              </p>
            </div>
          </div>

          <div className="flex shrink-0 flex-col items-end gap-1.5">
            <span className="text-[10px] font-semibold tracking-[0.2em] text-muted-foreground uppercase">
              Mode
            </span>
            <span
              className={cn(
                "rounded-full px-4 py-1.5 text-base font-semibold tracking-wide ring-1",
                isReadMode
                  ? "bg-sky-500/10 text-sky-700 ring-sky-500/20 dark:text-sky-300"
                  : "bg-amber-500/10 text-amber-700 ring-amber-500/20 dark:text-amber-300",
              )}
            >
              {module.state.mode.toUpperCase()}
            </span>
          </div>
        </div>
      </CardHeader>

      <CardContent className="grid gap-6 pt-6 lg:grid-cols-2">
        <section className="flex min-h-80 flex-col rounded-2xl bg-muted/35 p-5 ring-1 ring-foreground/8">
          <div className="flex items-center justify-between gap-3">
            <div>
              <p className="font-heading text-base font-medium">Current scan</p>
              <p className="mt-0.5 text-xs text-muted-foreground">
                A scan remains visible for 12 seconds
              </p>
            </div>
            <span className="relative flex size-3">
              {isReadMode && (
                <span className="absolute inline-flex size-full animate-ping rounded-full bg-emerald-400 opacity-60" />
              )}
              <span
                className={cn(
                  "relative inline-flex size-3 rounded-full",
                  isReadMode ? "bg-emerald-500" : "bg-muted-foreground/35",
                )}
              />
            </span>
          </div>

          <div className="mt-5 flex flex-1 flex-col justify-center">
            {currentScan ? (
              <div className="space-y-3" aria-live="polite">
                <div className="rounded-xl bg-background/80 p-4 ring-1 ring-foreground/10">
                  <p className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground uppercase">
                    Card UID
                  </p>
                  <p className="mt-2 break-all font-mono text-lg font-medium">
                    {currentScan.card_uid}
                  </p>
                </div>
                <div className="rounded-xl bg-background/80 p-4 ring-1 ring-foreground/10">
                  <p className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground uppercase">
                    Card data
                  </p>
                  <p className="mt-2 break-all font-mono text-sm text-foreground/80">
                    {currentScan.card_data || "No data stored on this card"}
                  </p>
                </div>
              </div>
            ) : (
              <div className="flex flex-col items-center py-8 text-center">
                <div className="grid size-16 place-items-center rounded-full border border-dashed border-muted-foreground/30 text-muted-foreground/55">
                  <CreditCard className="size-7" />
                </div>
                <p className="mt-4 font-medium">Waiting for a card</p>
                <p className="mt-1 max-w-64 text-sm text-muted-foreground">
                  Hold an RFID card near the reader to see its identification and data.
                </p>
              </div>
            )}
          </div>

          <Button
            type="button"
            variant="outline"
            className="mt-4 w-fit"
            disabled={disabled || isReadMode}
            onClick={() => setMode("Read")}
          >
            Switch to read
          </Button>
        </section>

        <section className="flex min-h-80 flex-col rounded-2xl bg-muted/35 p-5 ring-1 ring-foreground/8">
          <div>
            <p className="font-heading text-base font-medium">Write identifier</p>
            <p className="mt-0.5 text-xs text-muted-foreground">
              UUID encoded as 16 bytes
            </p>
          </div>

          <div className="mt-5">
            <label
              htmlFor={`rfid-id-${module.id}`}
              className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground uppercase"
            >
              Generated ID
            </label>
            <div className="mt-2 flex gap-2">
              <Input
                id={`rfid-id-${module.id}`}
                value={writableId.value}
                readOnly
                className="h-10 rounded-xl bg-background/80 font-mono text-xs"
              />
              <Button
                type="button"
                size="icon-lg"
                variant="outline"
                className="rounded-xl"
                aria-label="Regenerate UUID"
                title="Regenerate UUID"
                disabled={disabled || isSending}
                onClick={regenerateId}
              >
                <RefreshCw />
              </Button>
            </div>
          </div>

          <div className="mt-4">
            <p className="text-[10px] font-semibold tracking-[0.16em] text-muted-foreground uppercase">
              Byte payload
            </p>
            <div className="mt-2 flex flex-wrap gap-1.5 rounded-xl bg-background/80 p-3 ring-1 ring-foreground/10">
              {writableId.bytes.map((byte, index) => (
                <span
                  key={`${index}-${byte}`}
                  className="rounded-md bg-muted px-1.5 py-1 font-mono text-[11px] text-muted-foreground"
                >
                  {byte.toString(16).padStart(2, "0").toUpperCase()}
                </span>
              ))}
            </div>
          </div>

          <div className="mt-auto pt-5">
            {writeMessage && (
              <div
                className={cn(
                  "mb-3 flex items-start gap-2 rounded-xl px-3 py-2.5 text-sm ring-1",
                  writeMessage.state === "Good"
                    ? "bg-emerald-500/10 text-emerald-700 ring-emerald-500/20 dark:text-emerald-300"
                    : "bg-destructive/10 text-destructive ring-destructive/20",
                )}
                role="status"
              >
                {writeMessage.state === "Good" ? (
                  <CheckCircle2 className="mt-0.5 size-4 shrink-0" />
                ) : (
                  <TriangleAlert className="mt-0.5 size-4 shrink-0" />
                )}
                <span>{writeMessage.info}</span>
              </div>
            )}

            <div className="flex flex-wrap gap-2">
              <Button
                type="button"
                className="min-w-28"
                disabled={disabled || isSending}
                onClick={() => void sendId()}
              >
                <Send />
                {isSending ? "Sending…" : "Send"}
              </Button>
              <Button
                type="button"
                variant="outline"
                disabled={disabled || module.state.mode === "Write"}
                onClick={() => setMode("Write")}
              >
                Switch to write
              </Button>
            </div>
          </div>
        </section>
      </CardContent>
    </Card>
  );
});
