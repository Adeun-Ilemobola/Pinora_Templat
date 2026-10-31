// Serializes `{ id, command }` and sends it via the `send_data` IPC command.
//
// Since Phase 2A commands are typed and validated against the Zod schemas before
// sending (PIN-TPL-020): an invalid command is rejected here instead of reaching the
// device. The Tauri side still forwards COM strings unvalidated.

import { invoke } from "@tauri-apps/api/core";
import {
  IncomingCommandSchema,
  SYSTEM_COMMAND_ID,
  type IncomingCommand as IncomingCommandType,
} from "./protocol/command";

export const IncomingCommand = async (command: IncomingCommandType) => {
  try {
    const parsed = IncomingCommandSchema.safeParse(command);
    if (!parsed.success) {
      throw new Error(`Invalid command for ${command.id}: ${parsed.error.message}`);
    }
    const jsonString: string = JSON.stringify(command);

    const result = await invoke("send_data", { data: jsonString });
    return result;
  } catch (error) {
    console.error("Error handling incoming command:", error);
    throw error;
  }
};

/** Asks the firmware to re-send its system snapshot, registrations and current state
 *  (protocol v1). Firmware older than v1 cannot parse it and only logs an error. */
export const requestRediscovery = () =>
  IncomingCommand({ id: SYSTEM_COMMAND_ID, command: { System: { Rediscover: {} } } });
