import { invoke } from "@tauri-apps/api/core"
import { listen, UnlistenFn } from "@tauri-apps/api/event"
import z from "zod"
import { create } from "zustand"
import { ModuleEventSchema } from "./ModuleEven"
import { RegistrationSchema } from "./ModuleDefinitionSchema"
import { useModuleStore } from "./ModuleStore"


 export type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error" | '';

export const PortConnectionScheme = z.object({
  port: z.string().min(1, "Please select a port"),
  baudRate: z.coerce.number().int().positive("Must be a positive integer"),
});
export type PortConnectionType = z.infer<typeof PortConnectionScheme>;

export const InComingMessageSchema = z.discriminatedUnion("type", [
    z.object({
        type: z.literal("Registration"),
        payload: RegistrationSchema,
    }),
    z.object({
        type: z.literal("ModuleEvent"),
        payload: ModuleEventSchema,
    }),
])

// Validate messages individually so one malformed serial message does not
// discard every otherwise-valid message received in the same batch.
export const InComingMessageBatchSchema = z.array(z.unknown())

export type InComingMessage = z.infer<typeof InComingMessageSchema>

export type ActivityLogEntry = {
    id: number
    receivedAt: number
    message: InComingMessage
}

const MAX_ACTIVITY_LOGS = 500
let nextActivityLogId = 0


interface ListenStore {
    portInfo: PortConnectionType
    // listPorts: string[]
    commitTime: Date | null
    status: ConnectionStatus
    error: string | null,
    connect: (port: string, baudRate: number) => Promise<boolean>;
    disconnect: () => Promise<void>;
    getPorts: () => Promise<string[]>;
    setPortInfo: (info: Partial<PortConnectionType>) => void;
    activityLogs: ActivityLogEntry[];
    clearActivityLogs: () => void;




    // -------
    Unlisten: {
        unlistenBatch: UnlistenFn | null
        error: UnlistenFn | null
        // parseError: UnlistenFn | null
    }
    ListenersErr: boolean;
    isListening: boolean;
    startListeners: () => Promise<void>;
    stopListeners: () => Promise<void>;




}




export const useListenStore = create<ListenStore>((set, get) => ({
    portInfo: { port: "", baudRate: 115200 },
    commitTime: null,
    error: null,
    status: "",
    Unlisten: {
        unlistenBatch: null,
        error: null
        // parseError: UnlistenFn | null
    },
    ListenersErr: false,
    isListening: false,
    activityLogs: [],
    clearActivityLogs: () => set({ activityLogs: [] }),

    async disconnect() {
        await get().stopListeners();

        try {
            await invoke("stop_runtime");
        } catch (e) {
            console.error("[ListenStore] failed to stop serial runtime:", e);
        }

        set({
            commitTime: null,
            portInfo: { port: "", baudRate: 115200 },
            error: null,
            status: "",
            Unlisten: {
                unlistenBatch: null,
                error: null
                // parseError: UnlistenFn | null
            },
            ListenersErr: false,
            isListening: false,

        });


    },

    connect: async (port, baudRate) => {
        const portInfo = { port, baudRate: baudRate ?? get().portInfo.baudRate };
        const valid = PortConnectionScheme.safeParse(portInfo);
        if (!valid.success) {
            // console.warn("[PortStore] connect validation failed:", valid.error.issues[0].message);
            set({ error: valid.error.issues[0].message, status: "error" });
            return false;
        }
        await useListenStore.getState().stopListeners();
        set({ commitTime: null })

        set({ status: "connecting", error: null, portInfo: valid.data });
        try {
            await invoke("start_serial_listener", {
                portName: valid.data.port,
                baudRate: valid.data.baudRate,
            });
            // console.info(`[PortStore] connected to ${valid.data.port} at ${valid.data.baudRate} baud`);
            set({ status: "connected", commitTime: new Date() });
            await useListenStore.getState().startListeners();

            return true;
        } catch (e: any) {
            console.error("[PortStore] connect failed:", String(e));
            set({ status: "error", error: String(e) });
            return false;
        }



    },

    async getPorts() {
         try {
            const result = await invoke<string[]>("list_serial_ports");
            // console.info("[PortStore] available ports:", result);
            set({  error: null });
            return result
        } catch (e) {
            console.error("[PortStore] failed to list ports:", e);
            set({ error: "Failed to list ports:" + e });
            return []
        }
    },

    async startListeners() {
        if (get().isListening) {
            return;
        }

        try {
            // const error = await listen<unknown>("serial_error", (event) => {
            //     console.error("[ListenStore] serial error:", event.payload);
            // });

            const unlistenBatch = await listen<unknown>("serial_batch", (event) => {
                const batch = InComingMessageBatchSchema.safeParse(event.payload);

                if (!batch.success) {
                    console.error(
                        "[ListenStore] serial_batch was not an array:",
                        z.prettifyError(batch.error),
                    );
                    return;
                }

                const moduleStore = useModuleStore.getState();

                for (const rawMessage of batch.data) {
                    const parsedMessage = InComingMessageSchema.safeParse(rawMessage);

                    if (!parsedMessage.success) {
                        console.error(
                            "[ListenStore] invalid incoming message:",
                            z.prettifyError(parsedMessage.error),
                        );
                        continue;
                    }

                    const message = parsedMessage.data;

                    set((state) => ({
                        activityLogs: [
                            ...state.activityLogs,
                            { id: nextActivityLogId++, receivedAt: Date.now(), message },
                        ].slice(-MAX_ACTIVITY_LOGS),
                    }));

                    switch (message.type) {
                        case "Registration":
                            moduleStore.registerModule(message.payload);
                            break;
                        case "ModuleEvent":
                            moduleStore.dispatchModuleEvent(message.payload);
                            break;
                    }
                }
            });

            set({
                Unlisten: { unlistenBatch, error:null },
                ListenersErr: false,
                isListening: true,
            });
        } catch (e) {
            console.error("[ListenStore] failed to start listeners:", e);
            set({ ListenersErr: true, isListening: false });
        }
    },

    async stopListeners() {
        const { unlistenBatch, error } = get().Unlisten;
        unlistenBatch?.();
        error?.();

        set({
            Unlisten: { unlistenBatch: null, error: null },
            ListenersErr: false,
            isListening: false,
        });
    },
    setPortInfo(info) {
         set((pre) => ({
            portInfo: {
                ...pre.portInfo,
                ...info,
            }
         }));
    },
}))
