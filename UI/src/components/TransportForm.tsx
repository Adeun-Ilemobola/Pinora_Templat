import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Cable } from "lucide-react";
import { useModuleFront } from "@/lib/Modulefront";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
  CardDescription,
  CardAction,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
const rates = ["115200", "230400", "460800", "921600"];

export function TransportForm() {
  const connect = useModuleFront((s) => s.Connect);
  const disconnect = useModuleFront((s) => s.Disconnect);
  const status = useModuleFront((s) => s.PortStat);
  const portError = useModuleFront((s) => s.PortError);
  const port = useModuleFront((s) => s.PortState);
  const [name, setName] = useState("");
  const [rate, setRate] = useState("115200");
  const [ports, setPorts] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const connected = status === "Connected";
  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      setPorts(await invoke<string[]>("get_available_ports"));
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }
  useEffect(() => {
    void refresh();
  }, []);
  async function changeConnection() {
    setBusy(true);
    setError(null);
    try {
      if (connected) await disconnect();
      else await connect(name, Number(rate));
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }
  return (
    <Card className="h-full">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Cable className="size-4" />
          Transport
        </CardTitle>
        <CardDescription>Serial connection to your device.</CardDescription>
        <CardAction>
          <Badge variant={connected ? "secondary" : "outline"}>{status}</Badge>
        </CardAction>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="flex items-center justify-between">
          <Badge variant="outline">Serial</Badge>
          <Button
            variant="ghost"
            size="sm"
            disabled={loading || busy || connected}
            onClick={refresh}
          >
            <RefreshCw className={loading ? "animate-spin" : ""} />
            Refresh ports
          </Button>
        </div>
        <div className="grid gap-4 sm:grid-cols-2">
          <div className="space-y-2">
            <Label htmlFor="serial-port">Port</Label>
            <Select
              items={ports.map((value) => ({ label: value, value }))}
              value={connected ? port.name : name}
              onValueChange={(v) => setName(v ?? "")}
              disabled={connected || busy || loading}
            >
              <SelectTrigger id="serial-port" className="w-full">
                <SelectValue placeholder="Select a port" />
              </SelectTrigger>
              <SelectContent>
                {ports.map((p) => (
                  <SelectItem key={p} value={p}>
                    {p}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
          <div className="space-y-2">
            <Label htmlFor="baud-rate">Baud rate</Label>
            <Select
              items={rates.map((value) => ({ label: value, value }))}
              value={connected ? String(port.rate) : rate}
              onValueChange={(v) => setRate(v ?? "115200")}
              disabled={connected || busy}
            >
              <SelectTrigger id="baud-rate" className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {rates.map((r) => (
                  <SelectItem key={r} value={r}>
                    {r}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
        {!loading && !ports.length && !connected && (
          <p className="text-xs text-muted-foreground">
            No serial ports found. Attach a device and refresh.
          </p>
        )}
        {(error || portError) && (
          <Alert variant="destructive">
            <AlertDescription className="break-words">
              {error || portError}
            </AlertDescription>
          </Alert>
        )}
        <Button
          variant={connected ? "outline" : "default"}
          disabled={busy || (!connected && (!name || !ports.includes(name)))}
          onClick={changeConnection}
        >
          {busy ? "Please wait…" : connected ? "Disconnect" : "Connect"}
        </Button>
      </CardContent>
    </Card>
  );
}
