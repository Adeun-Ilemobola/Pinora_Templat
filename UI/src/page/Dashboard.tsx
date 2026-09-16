import { RegisteredButtonView } from "@/lib/Modules/button";
import { TransportForm } from "@/components/TransportForm";
import { Esp32StatsCard } from "@/components/Esp32StatsCard";
import { Card, CardContent } from "@/components/ui/card";
import { useModuleFront } from "@/lib/Modulefront";
// import { RegisteredLedView } from "@/lib/Modules/Led";

export default function Dashboard() {
  const registry = useModuleFront((state) => state.ModuleRegistry);
  const connected = useModuleFront((state) => state.PortStat === "Connected");
  // Registry replacement triggers discovery; each LED subscribes to its own store.
  

 
  const otherModules = Object.entries(registry).filter(([, store]) =>
    [
      "Button",
      "Led",
      
    ].includes(store.getState().kind),
  );
  const moduleCount = otherModules.length;

  return (
    <>
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Dashboard</h1>
        <p className="mt-1 text-sm text-muted-foreground">
          Device connection, system health, and module controls.
        </p>
      </div>
      <div className="grid items-stretch gap-6 lg:grid-cols-2">
        <TransportForm />
        <Esp32StatsCard />
      </div>
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
                  // case "Led":
                  //   return <RegisteredLedView key={id} moduleId={id} />;
                  default:
                    return null;
                }
              })}
             
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
