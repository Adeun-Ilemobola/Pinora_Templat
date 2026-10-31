import { z } from "zod";

export const RemoteButtonSchema = z.enum([
  "None",
  "Power",
  "VolumeUp",
  "FunctionStop",
  "Previous",
  "PlayPause",
  "Next",
  "Down",
  "VolumeDown",
  "Up",
  "Zero",
  "Equalizer",
  "StopRepeat",
  "One",
  "Two",
  "Three",
  "Four",
  "Five",
  "Six",
  "Seven",
  "Eight",
  "Nine",
]);

export type RemoteButton = z.infer<typeof RemoteButtonSchema>;

export const RemoteButtonEventSchema = z.union([
  z.object({
    Click: z.object({
      key: RemoteButtonSchema,
    }),
  }),
]);

export type RemoteButtonEvent = z.infer<typeof RemoteButtonEventSchema>;
