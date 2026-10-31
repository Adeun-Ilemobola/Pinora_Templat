import { Cpu, TriangleAlert } from "lucide-react";
import { useModuleFront } from "@/lib/Modulefront";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { PROTOCOL_VERSION, protocolVersionOf, type SystemInfo } from "@/lib/protocol/system";
const fields: [Exclude<keyof SystemInfo, "protocol_version">, string][] = [
  ["total_heap", "Total heap"],
  ["current_free_heap", "Free heap"],
  ["lowest_free_heap", "Lowest free heap"],
  ["largest_allocation", "Largest allocation"],
  ["maximum_app_slot", "Maximum app slot"],
  ["flash", "Flash"],
];

export function Esp32StatsCard() {
  const connected = useModuleFront((s) => s.PortStat === "Connected");
  const info = useModuleFront((s) => s.systemInfo);
  const duplicates = useModuleFront((s) => s.DuplicateLookUpIds);
  const version = info ? protocolVersionOf(info) : null;
  return (
    <Card className="h-full min-h-72">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Cpu className="size-4" />
          ESP32 statistics
        </CardTitle>
        <CardDescription>
          {connected && info
            ? `ESP-IDF ${info.esp_idf_version} · protocol v${version}`
            : "Memory and firmware reported by your device."}
        </CardDescription>
      </CardHeader>
      <CardContent className="flex flex-1 flex-col justify-center">
        {!connected ? (
          <p className="py-8 text-center text-sm text-muted-foreground">
            Connect to an ESP32 to view system statistics.
          </p>
        ) : !info ? (
          <div role="status" className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Waiting for system statistics…
            </p>
            <div className="grid grid-cols-2 gap-4">
              {fields.map(([key]) => (
                <Skeleton key={key} className="h-10 w-full" />
              ))}
            </div>
          </div>
        ) : (
          <div className="space-y-4">
            {version !== PROTOCOL_VERSION && (
              <p role="alert" className="flex items-start gap-2 text-sm text-destructive">
                <TriangleAlert className="mt-0.5 size-4 shrink-0" />
                Firmware speaks protocol v{version}; this app expects v{PROTOCOL_VERSION}.
                Some modules may not work until the firmware and app match.
              </p>
            )}
            {duplicates.length > 0 && (
              <p role="alert" className="flex items-start gap-2 text-sm text-destructive">
                <TriangleAlert className="mt-0.5 size-4 shrink-0" />
                Several modules share a lookup name ({duplicates.join(", ")}); only one of
                each is shown.
              </p>
            )}
            <dl className="grid grid-cols-2 gap-x-6 gap-y-4">
              {fields.map(([key, label]) => (
                <div key={key} className="min-w-0">
                  <dt className="text-xs text-muted-foreground">{label}</dt>
                  <dd className="mt-1 break-words font-mono text-sm font-medium">
                    {info[key]}
                  </dd>
                </div>
              ))}
            </dl>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
