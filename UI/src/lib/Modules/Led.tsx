
import { createStore, StoreApi } from 'zustand/vanilla'
import { useEffect, useState } from 'react';
import { useStore } from 'zustand';
import { useModuleFront } from '../Modulefront';
import { IncomingCommand } from '../IncomingCommand';
import { ModuleCard } from '@/components/ModuleCard';
import { Slider } from '@/components/ui/slider';
import { Badge } from '@/components/ui/badge';
import { Button } from "@/components/ui/button"

import { z } from "zod";

export const LedEventSchema = z.union([
    z.strictObject({
        Brightness: z.strictObject({
            level: z.number(),
        }),
    }),
]);

export type LedEvent = z.infer<typeof LedEventSchema>;

export const LedCommandSchema = z.union([
    z.strictObject({
        SetState: z.strictObject({
            state: z.number(),
        }),
    }),

    z.strictObject({
        Toggle: z.strictObject({}),
    }),
]);

export type LedCommand = z.infer<typeof LedCommandSchema>;
type LedInstance = {
    id: string;
    kind: string;
    state: number;
    look_up_id: string;

}

export interface LedModule extends LedInstance {
    toggle: () => void
    setBrightness: (v: number) => void
    handleEvent: (event: any) => void
}



export function createLed(data: LedInstance): StoreApi<LedModule> {
    return createStore<LedModule>((set) => ({
        id: data.id,
        kind: "Led",
        state: data.state,
        look_up_id: data.look_up_id,
        toggle: () => {
            const payload = {
                id: data.id,
                command: {
                    Led: {
                        Toggle: {}
                    }
                }
            }
            IncomingCommand(payload);

        },
        setBrightness: (v) => {
            const payload = {
                id: data.id,
                command:{
                    Led: {
                        SetState:{ state: v }
                    }
                }
            }
            IncomingCommand(payload);
        },
        handleEvent: (event : LedEvent) => {
            const [key, value] = Object.entries(event)[0] ;
            // Handle incoming events for the LED module

            if (key === "Brightness") {
                console.log("[LedModule - CORE] Brightness level:", value.level);
                set({ state: value.level });
            }

        },
    }))
}

export const LedView = ({ id }: { id: string }) => {
    const moduleId = useModuleFront(
        state => state.LookUpId[id]
    );

    if (!moduleId) {
        return <div>Waiting for LED {id}...</div>;
    }

    return <RegisteredLedView moduleId={moduleId} />;
};


export const RegisteredLedView = ({ moduleId }: { moduleId: string }) => {
    const store = useModuleFront(state => state.ModuleRegistry[moduleId]);
    if (!store || store.getState().kind !== "Led") return null;
    return <LedControls led={store as StoreApi<LedModule>} />;
};

function LedControls({ led }: { led: StoreApi<LedModule> }) {
    const ledId = useStore(led, state => state.id);
    const value = useStore(led, state => state.state);
    const toggle = useStore(led, state => state.toggle);
    const setBrightness = useStore(led, state => state.setBrightness);
    const connected = useModuleFront(state => state.PortStat === "Connected");
    const [draft, setDraft] = useState(value);
    useEffect(() => setDraft(value), [value]);
    return <ModuleCard type="LED" id={ledId}>
        <div className="space-y-5">
            <div className="flex items-center justify-between"><div><p className="text-xs text-muted-foreground">Reported brightness</p><p className="mt-1 text-2xl font-semibold tabular-nums">{value}<span className="ml-1 text-sm text-muted-foreground">%</span></p></div><Badge variant={value > 0 ? "secondary" : "outline"}>{value > 0 ? "On" : "Off"}</Badge></div>
            {/* Commit once per gesture; device events remain the source of reported state. */}
            <div className="space-y-3"><div className="flex justify-between text-xs text-muted-foreground"><span>Set brightness</span><span>{draft}%</span></div><Slider aria-label={`Brightness for LED ${ledId}`} min={0} max={100} step={1} value={[draft]} disabled={!connected} onValueChange={v => setDraft(Array.isArray(v) ? v[0] : v)} onValueCommitted={v => setBrightness(Array.isArray(v) ? v[0] : v)} /></div>
            <Button variant="outline" disabled={!connected} onClick={toggle}>Toggle LED</Button>
        </div>
    </ModuleCard>;
}
