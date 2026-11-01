import { z } from "zod";

// Mirrors pinora-protocol `PROTOCOL_VERSION`; checked against the golden fixture.
export const PROTOCOL_VERSION = 2;

export const SystemInfoSchema = z.object({
    // Absent before protocol v1 (Phase 2A); such firmware is treated as version 0.
    protocol_version: z.number().int().nonnegative().optional(),
    esp_idf_version: z.string(),
    total_heap: z.string(),
    current_free_heap: z.string(),
    lowest_free_heap: z.string(),
    largest_allocation: z.string(),
    maximum_app_slot: z.string(),
    flash: z.string(),
});

export type SystemInfo = z.infer<
    typeof SystemInfoSchema
>;

/** The firmware's protocol version; firmware older than v1 does not report one. */
export function protocolVersionOf(info: SystemInfo): number {
    return info.protocol_version ?? 0;
}
