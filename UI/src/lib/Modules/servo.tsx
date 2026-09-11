import { z } from "zod";
import { LedEvent, RegisteredLedView } from "./Led";
import { StoreApi, createStore } from "zustand/vanilla";
import { IncomingCommand } from "../IncomingCommand";
import { ModuleCard } from "@/components/ModuleCard";
import { useEffect, useState } from "react";
import { useStore } from "zustand";
import { useModuleFront } from "../Modulefront";
import { PivotSlider } from "@/components/PivotSlider";

export const ServoCapabilitySchema = z.object({
  max_angle: z.number(),
  min_angle: z.number(),
  offset: z.number(),
  min_pivot: z.number(),
  max_pivot: z.number(),
  pulse_min: z.number(),
  pulse_max: z.number(),
});

export type ServoCapability = z.infer<typeof ServoCapabilitySchema>;

export const ServoEventSchema = z.union([
  z.object({
    GetAngle: z.object({
      angle: z.number(),
    }),
  }),

  z.object({
    GetMinPivot: z.object({
      min_pivot: z.number(),
    }),
  }),

  z.object({
    GetMaxPivot: z.object({
      max_pivot: z.number(),
    }),
  }),

  z.object({
    GetOffset: z.object({
      angle: z.number(),
    }),
  }),
]);

export type ServoEvent = z.infer<typeof ServoEventSchema>;

export const ServoCommandSchema = z.union([
  z.object({
    SetAngle: z.object({
      angle: z.number(),
    }),
  }),

  z.object({
    SetMinPivot: z.object({
      min_pivot: z.number(),
    }),
  }),

  z.object({
    SetMaxPivot: z.object({
      max_pivot: z.number(),
    }),
  }),
]);

export type ServoCommand = z.infer<typeof ServoCommandSchema>;

type ServoInstance = {
  id: string;
  kind: string;
  look_up_id: string;
  state: {
    MinPivot: number;
    MaxPivot: number;
    Offset: number;
    Angle: number;
  };
};

export interface ServoModule extends ServoInstance {
  setAngle: (v: number) => void;
  setMinPivot: (v: number) => void;
  setMaxPivot: (v: number) => void;
  handleEvent: (event: any) => void;
}

export function createServo(data: ServoInstance): StoreApi<ServoModule> {
  return createStore<ServoModule>((set, get) => ({
    kind: "Servo",
    state: {
      MinPivot: data.state.MinPivot,
      MaxPivot: data.state.MaxPivot,
      Offset: data.state.Offset,
      Angle: data.state.Angle,
    },
    look_up_id: data.look_up_id,
    id: data.id,

    setAngle: (v) => {
      const payload = {
        id: data.id,
        command: {
          Servo: {
            SetAngle: { angle: v },
          },
        },
      };
      IncomingCommand(payload);
    },
    setMinPivot: (v) => {
      const payload = {
        id: data.id,
        command: {
          Servo: {
            SetMinPivot: { min_pivot: v },
          },
        },
      };
      IncomingCommand(payload);
    },
    setMaxPivot: (v) => {
      const payload = {
        id: data.id,
        command: {
          Servo: {
            SetMaxPivot: { max_pivot: v },
          },
        },
      };
      IncomingCommand(payload);
    },
    // setOffset: (v) => {
    //   const payload = {
    //     id: data.id,
    //     command: {
    //       Servo: {
    //         SetOffset: { offset: v },
    //       },
    //     },
    //   };
    //   IncomingCommand(payload);
    // },
    handleEvent: (event: ServoEvent) => {
      // Handle incoming events for the Servo module
      const [key, value] = Object.entries(event)[0];

      if (key === "GetAngle") {
        set({ state: { ...get().state, Angle: value.angle } });
      }
      if (key === "GetMinPivot") {
        set({ state: { ...get().state, MinPivot: value.min_pivot } });
      }
      if (key === "GetMaxPivot") {
        set({ state: { ...get().state, MaxPivot: value.max_pivot } });
      }
      if (key === "GetOffset") {
        set({ state: { ...get().state, Offset: value.angle } });
      }
    },
  }));
}

export const ServoView = ({ id }: { id: string }) => {
  const moduleId = useModuleFront((state) => state.LookUpId[id]);

  if (!moduleId) {
    return <div>Waiting for Servo {id}...</div>;
  }

  return <RegisteredServoView moduleId={moduleId} />;
};

export const RegisteredServoView = ({ moduleId }: { moduleId: string }) => {
  const store = useModuleFront((state) => state.ModuleRegistry[moduleId]);
  if (!store || store.getState().kind !== "Servo") return null;
  return <ServoControls servo={store as StoreApi<ServoModule>} />;
};

function ServoControls({ servo }: { servo: StoreApi<ServoModule> }) {
  const servoId = useStore(servo, (state) => state.id);
  const state = useStore(servo, (state) => state.state);
  const setAngle = useStore(servo, (state) => state.setAngle);
  const setMinPivot = useStore(servo, (state) => state.setMinPivot);
  const setMaxPivot = useStore(servo, (state) => state.setMaxPivot);
  const [draft, setDraft] = useState(state);
  useEffect(() => setDraft(state), [state]);
  return (
    <ModuleCard type="Servo" id={servoId}>
      <h1>Servo {state.Angle}</h1>

      <PivotSlider
        value={state.Angle}
        label="Angle"
        max={90}
        min={-90}
        onValueChange={(v) => {
          setAngle(v);
        }}
        showValue
        step={1}
      />
    </ModuleCard>
  );
}
