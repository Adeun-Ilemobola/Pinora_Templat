import z from "zod";
import { moduleTypeIdentifier, type ModuleDefinitionType } from "@shared/Protocol/ModuleDefinitionSchema";
export const WriteState = z.enum([
    "Good", "Bad"
])
export type WriteStateType = z.infer<typeof WriteState>

export const RfidMode = z.enum([
    "Read", "Write"
])

export type RfidModeType = z.infer<typeof RfidMode>



export const RfidModule = z.object({
    id: z.string(),
    lool_up_id: z.string(),
    parent_id: z.string(),
    module_type: z.literal(moduleTypeIdentifier.enum.Rfid),
    state: z.object({
        mode: RfidMode,
        writeMsg: z.object({
            state: WriteState,
            info: z.string(),
        }).nullable(),
        recentScan: z.object({
            card_uid: z.string(), card_data: z.string()
        }).nullable()

    }),
    mutableStateFields: z.tuple([z.literal("writeMsg"), z.literal("recentScan")]),
});
export type RfidModuleDefinition = z.infer<typeof RfidModule>;


export function RfidInitialBuild(
    id: string,
    parent_id: string,
    lool_up_id: string,
): RfidModuleDefinition {
    return {
        id,
        parent_id,
        module_type: moduleTypeIdentifier.enum.Rfid,
        lool_up_id,
        state: {
            recentScan: null,
            writeMsg: null,
            mode: RfidMode.enum.Read
        },
        mutableStateFields: ["writeMsg", "recentScan"],
    };
}

export const RfidCommandTypeSchema = z.discriminatedUnion("command", [
    z.object({
        command: z.literal("WriteMode"),

    }),

    z.object({
        command: z.literal("ReadMode"),

    }),
    z.object({
        command: z.literal("WritePayload"),
        data: z.array(z.number())
    }),


])
export const RfidEventSchema = z.discriminatedUnion("event_type", [
    z.object({
        event_type: z.literal("GetCard"),
        id: z.string(),
        card_uid: z.string(),
        card_data: z.string()
    }),
    z.object({
        event_type: z.literal("GetMode"),
        id: z.string(),
        mode: RfidMode,
    }),

    z.object({
        event_type: z.literal("GetWriteState"),
        id: z.string(),
        state: WriteState,
        info: z.string(),
    }),

]);

export function updateRfid(
    module: ModuleDefinitionType,
    event: z.infer<typeof RfidEventSchema>,
): ModuleDefinitionType {
    if (module.module_type !== moduleTypeIdentifier.enum.Rfid) return module;

    switch (event.event_type) {
        case "GetCard":
            return {
                ...module,
                state: {
                    ...module.state,
                    recentScan: {
                        card_uid: event.card_uid,
                        card_data: event.card_data
                    }
                }
            }


        case "GetMode":
            return {
                ...module,
                state: {
                    ...module.state,
                    mode: event.mode
                }
            }


        case "GetWriteState":
            return {
                ...module,
                state: {
                    ...module.state,
                    writeMsg: {
                        state: event.state,
                        info: event.info,
                    },
                }
            }

    }
}
