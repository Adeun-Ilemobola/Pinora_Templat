import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Command } from "@/lib/ModuleCommand";
import { RangefinderModule } from "@/lib/ModuleDefinitionSchema";
import { memo, useEffect, useState } from "react";
import z from "zod";
import ModuleCore from "./ModuleCore";

type RangefinderCardProps = {
  module: z.infer<typeof RangefinderModule>;
  sendCommand: (command: Command) => Promise<void>;
};

export const RangefinderCard = memo(function RangefinderCard({
  module,
  sendCommand,
}: RangefinderCardProps) {
  const [timingBudget, setTimingBudget] = useState(
    module.state.timing_budget_ms,
  );

  useEffect(() => {
    setTimingBudget(module.state.timing_budget_ms);
  }, [module.state.timing_budget_ms]);

  function startRanging() {
    void sendCommand({
      id: module.id,
      module_type: "Rangefinder",
      payload: {
        command: "StartRanging",
      },
    });
  }

  function stopRanging() {
    void sendCommand({
      id: module.id,
      module_type: "Rangefinder",
      payload: {
        command: "StopRanging",
      },
    });
  }

  function setMeasurementTimingBudget() {
    const milliseconds = Math.round(timingBudget);

    if (!Number.isFinite(milliseconds) || milliseconds <= 0) {
      return;
    }

    void sendCommand({
      id: module.id,
      module_type: "Rangefinder",
      payload: {
        command: "SetTimingBudget",
        milliseconds,
      },
    });
  }

  function setDistanceMode(mode: "Short" | "Long") {
    void sendCommand({
      id: module.id,
      module_type: "Rangefinder",
      payload: {
        command: "SetDistanceMode",
        mode,
      },
    });
  }

  return (
    <ModuleCore
      id={module.id}
      manuel_id={module.lool_up_id}
      moduletype={module.module_type}
    >
      <div className="flex w-full flex-col gap-5">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <p className="text-xs text-accent-foreground/35">Range</p>

            <div className="flex items-baseline gap-2">
              <h1 className="text-3xl font-medium">
                {module.state.range_mm}
              </h1>
              <span className="text-sm text-accent-foreground/50">mm</span>
            </div>
          </div>

          <div className="flex items-center gap-2.5">
            <span
              className={
                "h-3 w-3 rounded-full " +
                (module.state.is_ranging
                  ? "bg-green-500 shadow-[0_0_8px_2px] shadow-green-500/60"
                  : "bg-gray-500")
              }
            />

            <span className="text-sm font-medium">
              {module.state.is_ranging ? "Ranging" : "Stopped"}
            </span>
          </div>
        </div>

        <div className="grid gap-3 sm:grid-cols-2">
          <div className="flex flex-col gap-1">
            <label className="text-xs text-accent-foreground/35">
              Measurement timing budget
            </label>

            <Input
              type="number"
              min={1}
              step={1}
              value={timingBudget}
              onChange={(event) => {
                setTimingBudget(Number(event.target.value));
              }}
              onKeyDown={(event) => {
                if (event.key !== "Enter") {
                  return;
                }

                event.preventDefault();
                setMeasurementTimingBudget();
              }}
            />
          </div>

          <div className="flex flex-col gap-1">
            <label className="text-xs text-accent-foreground/35">
              Distance mode
            </label>

            <div className="flex gap-2">
              <Button
                type="button"
                className="flex-1"
                variant={
                  module.state.distance_mode === "Short"
                    ? "default"
                    : "outline"
                }
                onClick={() => setDistanceMode("Short")}
              >
                Short
              </Button>

              <Button
                type="button"
                className="flex-1"
                variant={
                  module.state.distance_mode === "Long"
                    ? "default"
                    : "outline"
                }
                onClick={() => setDistanceMode("Long")}
              >
                Long
              </Button>
            </div>
          </div>
        </div>

        {module.state.last_invalid_status !== null && (
          <div className="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2">
            <p className="text-xs text-destructive">
              Invalid measurement: {module.state.last_invalid_status}
            </p>
          </div>
        )}

        <div className="flex flex-wrap gap-2.5">
          <Button
            disabled={module.state.is_ranging}
            onClick={startRanging}
          >
            Start Ranging
          </Button>

          <Button
            variant="destructive"
            disabled={!module.state.is_ranging}
            onClick={stopRanging}
          >
            Stop Ranging
          </Button>

          <Button
            variant="outline"
            onClick={setMeasurementTimingBudget}
          >
            Set Timing Budget
          </Button>
        </div>
      </div>
    </ModuleCore>
  );
});