import { z } from "zod";

export const SystemInfoSchema = z.object({
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