import { ShieldAlert, ShieldCheck, ShieldQuestion } from "lucide-react";
import { useModuleFront } from "@/lib/Modulefront";
import { describeAuth, type AuthTone } from "@/lib/auth";
import { protocolVersionOf } from "@/lib/protocol/system";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

const toneClass: Record<AuthTone, string> = {
  ok: "text-emerald-600 dark:text-emerald-400",
  info: "text-muted-foreground",
  warning: "text-amber-600 dark:text-amber-400",
  error: "text-destructive",
};

// Device authentication status and USB pairing (audit Phase 2B, PIN-TPL-003).
export function DeviceSecurityCard() {
  const connected = useModuleFront((s) => s.PortStat === "Connected");
  const transport = useModuleFront((s) => s.TransportMode);
  const auth = useModuleFront((s) => s.DeviceAuth);
  const info = useModuleFront((s) => s.systemInfo);
  const pair = useModuleFront((s) => s.PairDevice);
  const unpair = useModuleFront((s) => s.UnpairDevice);
  const error = useModuleFront((s) => s.PairingError);

  if (!connected) return null;
  const summary = describeAuth(auth, transport, info ? protocolVersionOf(info) : null);
  const Icon = summary.tone === "ok" ? ShieldCheck : summary.tone === "info" ? ShieldQuestion : ShieldAlert;
  const overUsb = transport === "Serial";

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Icon className={`size-4 ${toneClass[summary.tone]}`} />
          Device security
        </CardTitle>
        <CardDescription className={toneClass[summary.tone]}>{summary.title}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <p className="text-sm text-muted-foreground">{summary.detail}</p>
        {overUsb && (
          <div className="flex flex-wrap gap-2">
            <Button onClick={() => void pair()}>Pair this device</Button>
            <Button variant="outline" onClick={() => void unpair()}>
              Clear device key
            </Button>
          </div>
        )}
        {error && (
          <p role="alert" className="text-sm text-destructive">
            {error}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
