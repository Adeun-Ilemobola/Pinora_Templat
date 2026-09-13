import { RegisteredButtonView } from "@/lib/Modules/button";
import { RegisteredImuView } from "@/lib/Modules/imu";
import { RegisteredRemoteReceiverView } from "@/lib/Modules/remote-receiver";
import { RegisteredRangefinderView } from "@/lib/Modules/rangefinder";
import { RegisteredRfidView } from "@/lib/Modules/rfid";
import { RegisteredStepperView } from "@/lib/Modules/stepper";
import { useMemo } from "react";
import { TransportForm } from "@/components/TransportForm";
import { Esp32StatsCard } from "@/components/Esp32StatsCard";
import { LogViewer } from "@/components/LogViewer";
import { Card, CardContent } from "@/components/ui/card";
import { useModuleFront } from "@/lib/Modulefront";
import { RegisteredLedView } from "@/lib/Modules/Led";
import { RegisteredServoView } from "@/lib/Modules/servo";

export default function Dashboard() {
  const registry = useModuleFront((state) => state.ModuleRegistry);
  const connected = useModuleFront((state) => state.PortStat === "Connected");
  // Registry replacement triggers discovery; each LED subscribes to its own store.
  const leds = useMemo(
    () =>
      Object.entries(registry).filter(
        ([, store]) => store.getState().kind === "Led",
      ),
    [registry],
  );

  const servos = useMemo(
    () =>
      Object.entries(registry).filter(
        ([, store]) => store.getState().kind === "Servo",
      ),
    [registry],
  );

  const otherModules = Object.entries(registry).filter(([, store]) =>
    [
      "Button",
      "Imu",
      "Rangefinder",
      "RemoteReceiver",
      "Rfid",
      "StepperMotor",
    ].includes(store.getState().kind),
  );
  const moduleCount = leds.length + servos.length + otherModules.length;

  return (
    <>
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Dashboard</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Device connection, protocol traffic, and module controls.
        </p>
      </div>
      <div className="grid items-stretch gap-6 lg:grid-cols-2">
        <TransportForm />
        <Esp32StatsCard />
      </div>
      <LogViewer />
      <section className="space-y-4" aria-labelledby="modules-heading">
        <div className="flex items-center justify-between">
          <h2 id="modules-heading" className="text-lg font-semibold">
            Modules
          </h2>
          <span className="text-xs text-muted-foreground">
            {connected ? moduleCount : 0} modules available
          </span>
        </div>
        {connected && moduleCount ? (
          <>
            <div className="grid gap-4 md:grid-cols-2 2xl:grid-cols-3">
              {otherModules.map(([id, store]) => {
                switch (store.getState().kind) {
                  case "Button":
                    return <RegisteredButtonView key={id} moduleId={id} />;
                  case "Imu":
                    return <RegisteredImuView key={id} moduleId={id} />;
                  case "RemoteReceiver":
                    return (
                      <RegisteredRemoteReceiverView key={id} moduleId={id} />
                    );
                  case "Rangefinder":
                    return <RegisteredRangefinderView key={id} moduleId={id} />;
                  case "Rfid":
                    return <RegisteredRfidView key={id} moduleId={id} />;
                  case "StepperMotor":
                    return <RegisteredStepperView key={id} moduleId={id} />;
                  default:
                    return null;
                }
              })}
              {leds.map(([id]) => (
                <RegisteredLedView key={id} moduleId={id} />
              ))}

              {servos.map(([id]) => (
                <RegisteredServoView key={id} moduleId={id} />
              ))}
            </div>
          </>
        ) : (
          <Card>
            <CardContent className="py-5 text-sm text-muted-foreground">
              {connected
                ? "Waiting for  module registration."
                : "Connect a device to access  modules."}
            </CardContent>
          </Card>
        )}
      </section>
    </>
  );
}
