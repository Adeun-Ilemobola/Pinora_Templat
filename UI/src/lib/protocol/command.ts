// Zod schemas for commands sent to the firmware (`IncomingCommand` in pinora-protocol).
//
// Every family is covered since Phase 2A (PIN-TPL-007, PIN-TPL-020); the golden
// fixture test (UI/tests/protocol-fixtures.test.ts) parses every `cmd.*` line of
// protocol/tests/golden/wire_format.txt with `IncomingCommandSchema`, so a Rust
// change that is not mirrored here fails the build.
//
// Family schemas are imported from `../Modules/<family>/schema` directly; the family
// index modules also export stores and views, which import this file (cycle).
import { z } from "zod";
import { LedCommandSchema } from "../Modules/Led/schema";
import { LidarCommandSchema } from "../Modules/lidar/schema";
import { RangefinderCommandSchema } from "../Modules/rangefinder/schema";
import { RfidCommandSchema } from "../Modules/rfid/schema";
import { ServoCommandSchema } from "../Modules/servo/schema";
import { StepperMotorCommandSchema } from "../Modules/stepper/schema";

/** Protocol v1 system commands, sent to the reserved id `SYSTEM_COMMAND_ID`. */
export const SystemCommandSchema = z.union([
  // Re-send the system snapshot, every registration and current module state.
  z.strictObject({ Rediscover: z.strictObject({}) }),
]);

export type SystemCommand = z.infer<typeof SystemCommandSchema>;

/** Reserved command id for system commands (contract C-05). */
export const SYSTEM_COMMAND_ID = "SSI";

export const ModuleCommandSchema = z.union([
  z.strictObject({ Led: LedCommandSchema }),
  z.strictObject({ Servo: ServoCommandSchema }),
  z.strictObject({ Lidar: LidarCommandSchema }),
  z.strictObject({ Rangefinder: RangefinderCommandSchema }),
  z.strictObject({ StepperMotor: StepperMotorCommandSchema }),
  z.strictObject({ Rfid: RfidCommandSchema }),
  z.strictObject({ System: SystemCommandSchema }),
]);

export type ModuleCommand = z.infer<typeof ModuleCommandSchema>;

export const IncomingCommandSchema = z.strictObject({
  id: z.string(),
  command: ModuleCommandSchema,
});

export type IncomingCommand = z.infer<typeof IncomingCommandSchema>;
