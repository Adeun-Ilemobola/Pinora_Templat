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
import { Input } from "@/components/ui/input";
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
  const connectWifi = useModuleFront((s) => s.ConnectWifi);
  const connectBluetooth = useModuleFront((s) => s.ConnectBluetooth);
  const disconnect = useModuleFront((s) => s.Disconnect);
  const status = useModuleFront((s) => s.PortStat);
  const activeMode = useModuleFront((s) => s.TransportMode);
  const portError = useModuleFront((s) => s.PortError);
  const port = useModuleFront((s) => s.PortState);
  const wifiPort = useModuleFront((s) => s.WifiPort);
  const refreshWifiStatus = useModuleFront((s) => s.RefreshWifiStatus);
  const [name, setName] = useState("");
  const [mode, setMode] = useState<"Serial" | "Wi-Fi" | "Bluetooth">("Serial");
  const [rate, setRate] = useState("115200");
  const [wifiPortInput, setWifiPortInput] = useState(wifiPort ? String(wifiPort) : "");
  const [ports, setPorts] = useState<string[]>([]);
  const [bluetoothPorts, setBluetoothPorts] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const connected = status === "Connected" || status === "Listening";
  const displayStatus = mode === "Wi-Fi" && status === "Disconnected" ? "Stopped" : status;
  const validWifiPort = Number.isInteger(Number(wifiPortInput)) &&
    Number(wifiPortInput) >= 1 && Number(wifiPortInput) <= 65535;
  async function refresh() {
    setLoading(true);
    setError(null);
    const [serial, bluetooth] = await Promise.allSettled([
      invoke<string[]>("get_available_ports"),
      invoke<string[]>("get_available_bluetooth_ports"),
    ]);
    setPorts(serial.status === "fulfilled" ? serial.value : []);
    setBluetoothPorts(bluetooth.status === "fulfilled" ? bluetooth.value : []);
    const errors = [
      serial.status === "rejected" && `Serial ports: ${String(serial.reason)}`,
      bluetooth.status === "rejected" && `Bluetooth ports: ${String(bluetooth.reason)}`,
    ].filter(Boolean);
    setError(errors.length ? errors.join("; ") : null);
    setLoading(false);
  }
  useEffect(() => {
    void refresh();
    void refreshWifiStatus().catch((err) => setError(String(err)));
  }, []);
  useEffect(() => {
    setWifiPortInput(wifiPort ? String(wifiPort) : "");
  }, [wifiPort]);
  useEffect(() => {
    if (activeMode) setMode(activeMode);
  }, [activeMode]);
  async function changeConnection() {
    setBusy(true);
    setError(null);
    try {
      if (connected) await disconnect();
      else if (mode === "Wi-Fi") await connectWifi(Number(wifiPortInput));
      else if (mode === "Bluetooth") await connectBluetooth(name);
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
        <CardDescription>Connect by USB serial, paired Bluetooth SPP, or Wi-Fi.</CardDescription>
        <CardAction>
          <Badge variant={connected ? "secondary" : "outline"}>{displayStatus}</Badge>
        </CardAction>
      </CardHeader>
      <CardContent className="space-y-5">
        <div className="flex items-center justify-between">
          <Badge variant="outline">{mode}</Badge>
          {mode !== "Wi-Fi" && (
          <Button
            variant="ghost"
            size="sm"
            disabled={loading || busy || connected}
            onClick={refresh}
          >
            <RefreshCw className={loading ? "animate-spin" : ""} />
            Refresh ports
          </Button>
          )}
        </div>
        <div className="space-y-2">
          <Label htmlFor="transport-mode">Transport</Label>
          <Select
            items={["Serial", "Wi-Fi", "Bluetooth"].map((value) => ({ label: value, value }))}
            value={mode}
            onValueChange={(value) => {
              setMode(value === "Wi-Fi" ? "Wi-Fi" : value === "Bluetooth" ? "Bluetooth" : "Serial");
              setName("");
            }}
            disabled={connected || busy}
          >
            <SelectTrigger id="transport-mode" className="w-full">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="Serial">Serial</SelectItem>
              <SelectItem value="Wi-Fi">Wi-Fi</SelectItem>
              <SelectItem value="Bluetooth">Bluetooth</SelectItem>
            </SelectContent>
          </Select>
        </div>
        {mode === "Serial" ? (
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
        ) : mode === "Bluetooth" ? (
          <div className="space-y-2">
            <Label htmlFor="bluetooth-port">Paired Bluetooth COM port</Label>
            <Select
              items={bluetoothPorts.map((value) => ({ label: value, value }))}
              value={connected ? port.name : name}
              onValueChange={(value) => setName(value ?? "")}
              disabled={connected || busy || loading}
            >
              <SelectTrigger id="bluetooth-port" className="w-full">
                <SelectValue placeholder="Select a paired SPP port" />
              </SelectTrigger>
              <SelectContent>
                {bluetoothPorts.map((value) => (
                  <SelectItem key={value} value={value}>{value}</SelectItem>
                ))}
              </SelectContent>
            </Select>
            {!loading && bluetoothPorts.length === 0 && (
              <p className="text-xs text-muted-foreground">
                Pair Pinora Dev ESP32 in Windows Bluetooth settings, then refresh ports.
                The USB CP210x port is not the Bluetooth port.
              </p>
            )}
          </div>
        ) : (
          <div className="space-y-2">
            <Label htmlFor="wifi-port">Port</Label>
            <Input
              id="wifi-port"
              type="number"
              min={1}
              max={65535}
              step={1}
              value={wifiPortInput}
              onChange={(event) => setWifiPortInput(event.target.value)}
              disabled={connected || busy}
            />
          </div>
        )}
        {mode === "Serial" && !loading && !ports.length && !connected && (
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
          disabled={busy || (mode === "Serial" && !connected && (!name || !ports.includes(name))) ||
            (mode === "Bluetooth" && !connected && (!name || !bluetoothPorts.includes(name))) ||
            (mode === "Wi-Fi" && !connected && !validWifiPort)}
          onClick={changeConnection}
        >
          {busy ? "Please wait…" : connected ? mode === "Wi-Fi" ? "Stop Listener" : "Disconnect" : mode === "Wi-Fi" ? "Start Listener" : "Connect"}
        </Button>
      </CardContent>
    </Card>
  );
}
