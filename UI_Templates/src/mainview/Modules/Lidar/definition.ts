import z from "zod";
import type { ModuleDefinitionType } from "@shared/Protocol/ModuleDefinitionSchema";

export const PointSchema = z.object({
  x: z.number().int().min(-90).max(90),
  y: z.number().int().min(-90).max(90),
});

export const RangePointSchema = z.object({
  x: z.number().int(),
  y: z.number().int(),
  distant: z.number().int().nonnegative(),
});

export const LidarCommandTypeSchema = z.discriminatedUnion("command", [
  z.object({
    command: z.literal("Roi"),
    min: PointSchema,
    max: PointSchema,
  }),
  z.object({
    command: z.literal("StartScan"),
  }),
  z.object({
    command: z.literal("StopScan"),
  }),
  z.object({
    command: z.literal("Test"),
  }),
  z.object({
    command: z.literal("SetStep"),
    step: z.number().int().positive(),
  }),
  z.object({
    command: z.literal("ChangeMotorAngle"),
    id: z.string(),
    step: z.number().int(),
  }),
  z.object({
    command: z.literal("MovePos"),
    p: PointSchema,
  }),
]);

export const LidarEventSchema = z.discriminatedUnion("event_type", [
  z.object({
    event_type: z.literal("Roi"),
    id: z.string(),
    min: PointSchema,
    max: PointSchema,
  }),
  z.object({
    event_type: z.literal("PointMap"),
    id: z.string(),
    max_chunk: z.number(),
    curr_chunk: z.number(),
    map: z.array(RangePointSchema),
  }),
  z.object({
    event_type: z.literal("Target"),
    id: z.string(),
    point: PointSchema,
  }),
  z.object({
    event_type: z.literal("ScanState"),
    id: z.string(),
    state: z.enum(["Idol", "Scanning", "StopScan"]),
    scan_time:z.number()
  }),
]);

export const LidarModule = z.object({
  id: z.string(),
  lool_up_id: z.string(),
  parent_id: z.string(),
  module_type: z.literal("Lidar"),
  state: z.object({
    state: z.enum(["Idol", "Scanning", "StopScan"]),
    map: z.array(RangePointSchema),
    ROI: z.object({
      min: PointSchema,
      max: PointSchema,
    }),
    scanTime:z.number()
  }),
  mutableStateFields: z.tuple([
    z.literal("map"),
  ]),
});

export type Point = z.infer<typeof PointSchema>;
export type RangePoint = z.infer<typeof RangePointSchema>;

export function lidarInitialBuild(
  id: string,
  parent_id: string,
  lool_up_id: string,
): z.infer<typeof LidarModule> {
  return {
    id,
    parent_id,
    module_type: "Lidar",
    lool_up_id,
    state: {
      state: "Idol",
      map: [],
      ROI: {
        min: { x: 0, y: 0 },
        max: { x: 0, y: 0 },
      },
      scanTime:0.0
    },
    mutableStateFields: [
      "map"
    ] as const,
  };
}

export function updateLidar(
  module: ModuleDefinitionType,
  event: z.infer<typeof LidarEventSchema>,
): ModuleDefinitionType {
  if (module.module_type !== "Lidar") return module;

  switch (event.event_type) {
    case "Roi":
      return {
        ...module,
        state: {
          ...module.state,
          ROI: { min: event.min, max: event.max },
        },
      };

    case "PointMap":
      // console.debug(
      //   `
      //    ---------------------------
      //   [ event_type : PointMap ]
      //    size : ${event.map.length}
      //    curr_chunk :${event.curr_chunk}
      //    max_chunk :${event.max_chunk}
      //    ---------------------------
      //   `
      //   ,
      //   event.map
      // )

      return {
        ...module,
        state: {
          ...module.state,
          map: [...module.state.map, ...event.map]
        },
      };

    case "Target":
      return module;

    case "ScanState":
      return {
        ...module,
        state: { 
          ...module.state, 
          state: event.state ,
          scanTime:event.scan_time
        },
      };
  }
}

export const lidarHeatmapColors = [
    "#30123B", // nearest
    "#4145AB",
    "#466BE3",
    "#3E8EED",
    "#2FA7D8",
    "#1BB9C4",
    "#18C7A3",
    "#2DD080",
    "#55D45F",
    "#7BD64A",
    "#A2D83D",
    "#C7D83B",
    "#E4D63A",
    "#F5CE38",
    "#FBC234",
    "#FBAE32",
    "#F89430",
    "#F4772E",
    "#EC5B2D",
    "#E34232",
    "#D62F3D",
    "#B51F2E",
    "#911722",
    "#6C1019",
    "#460B12", // farthest
] as const;


export const roiBorder = "#FFFFFF";
export const GRID_BACKGROUND_COLOUR = "#101014";
export const DEFAULT_CELL_COLOUR = "#29292F";

export const MIN_VALID_LIDAR_DISTANCE_MM = 100;
export const MAX_VALID_LIDAR_DISTANCE_MM = 4000;

export const LIDAR_DISPLAY_MIN_MM = 100;
export const LIDAR_DISPLAY_MAX_MM = 4000;

export const LIDAR_COLOUR_LEVELS = 64;
export const LIDAR_CONTRAST = 1.0;

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

function applyContrast(value: number, contrast: number): number {
  const t = clamp(value, 0, 1);

  if (t < 0.5) {
    return 0.5 * Math.pow(t * 2, contrast);
  }

  return 1 - 0.5 * Math.pow((1 - t) * 2, contrast);
}

function hexToRgb(hex: string): [number, number, number] {
  const value = Number.parseInt(hex.slice(1), 16);

  return [
    (value >> 16) & 255,
    (value >> 8) & 255,
    value & 255,
  ];
}

function rgbToHex(red: number, green: number, blue: number): string {
  return `#${[red, green, blue]
    .map(channel =>
      clamp(Math.round(channel), 0, 255)
        .toString(16)
        .padStart(2, "0"),
    )
    .join("")}`;
}

function samplePalette(position: number): string {
  const t = clamp(position, 0, 1);
  const scaledIndex = t * (lidarHeatmapColors.length - 1);

  const lowerIndex = Math.floor(scaledIndex);
  const upperIndex = Math.min(
    lowerIndex + 1,
    lidarHeatmapColors.length - 1,
  );

  const mix = scaledIndex - lowerIndex;

  const lower = hexToRgb(lidarHeatmapColors[lowerIndex]);
  const upper = hexToRgb(lidarHeatmapColors[upperIndex]);

  return rgbToHex(
    lower[0] + (upper[0] - lower[0]) * mix,
    lower[1] + (upper[1] - lower[1]) * mix,
    lower[2] + (upper[2] - lower[2]) * mix,
  );
}

export const INVALID_READING_COLOUR = "#29292F";
export const OUT_OF_RANGE_COLOUR = "#6B7280";

export function distanceToColour(
    mm: number,
    minMm = LIDAR_DISPLAY_MIN_MM,
    maxMm = LIDAR_DISPLAY_MAX_MM,
): string {
    if (!Number.isFinite(mm) || mm <= 0) {
        return INVALID_READING_COLOUR;
    }

    if (mm > MAX_VALID_LIDAR_DISTANCE_MM) {
        return OUT_OF_RANGE_COLOUR;
    }

    let normalized = (mm - minMm) / (maxMm - minMm);
    normalized = clamp(normalized, 0, 1);

    if (LIDAR_CONTRAST !== 1) {
        normalized = applyContrast(normalized, LIDAR_CONTRAST);
    }

    const quantized =
        Math.round(normalized * (LIDAR_COLOUR_LEVELS - 1)) /
        (LIDAR_COLOUR_LEVELS - 1);

    return samplePalette(quantized);
}