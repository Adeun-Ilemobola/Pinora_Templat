import { z } from "zod";

import { createStore, StoreApi } from "zustand/vanilla";

export const RangePointSchema = z.object({
    x: z.number(),
    y: z.number(),
    distant: z.number(),
});

export type RangePoint = z.infer<typeof RangePointSchema>;

export const PointSchema = z.object({
    x: z.number(),
    y: z.number(),
});

export type Point = z.infer<typeof PointSchema>;

export const ScanStateSchema = z.enum([
    "Idol",
    "Scanning",
    "StopScan",
]);

export type ScanState = z.infer<typeof ScanStateSchema>;

export const LidarEventSchema = z.union([
    z.object({
        Roi: z.object({
            min: PointSchema,
            max: PointSchema,
        }),
    }),

    z.object({
        PointMap: z.object({
            max_chunk: z.number(),
            curr_chunk: z.number(),
            map: z.array(RangePointSchema),
        }),
    }),

    z.object({
        Target: z.object({
            point: PointSchema,
        }),
    }),

    z.object({
        ScanState: z.object({
            state: ScanStateSchema,
            scan_time: z.number(),
        }),
    }),
]);

export type LidarEvent = z.infer<typeof LidarEventSchema>;

export const LidarCommandSchema = z.union([
    z.object({
        Roi: z.object({
            min: PointSchema,
            max: PointSchema,
        }),
    }),

    z.literal("StartScan"),
    z.literal("StopScan"),
    z.literal("Test"),

    z.object({
        SetStep: z.object({
            step: z.number(),
        }),
    }),

    z.object({
        ChangeMotorAngle: z.object({
            id: z.string(),
            step: z.number(),
        }),
    }),

    z.object({
        MovePos: z.object({
            p: PointSchema,
        }),
    }),
]);

export type LidarCommand = z.infer<
    typeof LidarCommandSchema
>;


type LidarInstance = {
    id: string;
    kind: string;
    look_up_id: string;


}

export interface LidarModule extends LidarInstance {
    handleEvent: (event: any) => void
}


export function createLidar(data: LidarInstance): StoreApi<LidarModule> {
    return createStore<LidarModule>(() => ({
        id: data.id,
        kind: "Lidar",
        look_up_id: data.look_up_id,
        handleEvent: (event) => {
            // Handle incoming events for the Lidar module
            console.log("Handling event for Lidar:", event);
        },
    }))
}
