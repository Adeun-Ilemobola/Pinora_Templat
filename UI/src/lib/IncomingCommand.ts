
import { invoke } from "@tauri-apps/api/core";

type IncomingCommandType = { id: string, command: any};
export const IncomingCommand = async (command: IncomingCommandType) => {
    try {
        const jsonString: string = JSON.stringify(command); 

        const result = await invoke("send_data", { data: jsonString });
        return result;
    } catch (error) {
        console.error("Error handling incoming command:", error);
        throw error;
    }
};