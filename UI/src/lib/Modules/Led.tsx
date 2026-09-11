
import { createStore, StoreApi } from 'zustand/vanilla'
import { getModule } from '../ModuleGeter';
import { useStore } from 'zustand';
import { useModuleFront } from '../Modulefront';
import { IncomingCommand } from '../IncomingCommand';
import { Card , CardContent , CardHeader } from '@/components/ui/card';
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


const RegisteredLedView = ({ moduleId }: { moduleId: string }) => {
    const led = getModule(moduleId, "Led") as StoreApi<LedModule>;

    const ledId = useStore(led, state => state.id);
    const value = useStore(led, state => state.state);
    const toggle = useStore(led, state => state.toggle);

    return (
        <Card className="w-40 h-fit">
            <CardHeader>
                LED ID: {ledId}
            </CardHeader>
            <CardContent className="w-32 h-fit">
                State: {value}
                <div>
                    <Button onClick={toggle}>
                        Toggle
                    </Button>
                </div>
            </CardContent>
        </Card>
    );
};