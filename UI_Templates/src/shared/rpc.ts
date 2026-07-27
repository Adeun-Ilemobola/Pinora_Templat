import type { RPCSchema } from "electrobun/bun";
import { InComingMessageSchema } from "./Protocol/ModuleDefinitionSchema";
import z from "zod";
import { Commandtype } from "./Protocol/ModuleCommand";
 type ConnectionStatus = "disconnected" | "connecting" | "connected" | "error" | '';

export type SerialDeviceInfo = {
  path: string;
  manufacturer: string | null;
  serialNumber: string | null;
  vendorId: string | null;
  productId: string | null;
  locationId: string | null;
  pnpId: string | null;
};

export type AppRPC = {
  bun: RPCSchema<{
    requests: {
      getAvailablePorts: {
        params: void;
        response: SerialDeviceInfo[];
      },
      openPort:{
        params:{
          port:string
        },
        response:{}
      }
      sendComand:{
        params:Commandtype,
        response:{}
      }
    };

    messages: {};
  }>;

  webview: RPCSchema<{
    requests: {};
    messages: {
      incomingMessage:{
        message:z.infer<typeof InComingMessageSchema>
      },
      PortStatus:{
        status:ConnectionStatus,
        path:string

      }
    };
  }>;
};