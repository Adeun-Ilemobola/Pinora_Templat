// src/mainview/electrobun.ts

import { Electroview } from "electrobun/view";
import type { AppRPC } from "@src/bun/rpc";
import { useModuleStore } from "@src/bun/Runtime/ModuleStore";


const rpc = Electroview.defineRPC<AppRPC>({
    handlers: {
        requests: {



        },
        messages: {
            incomingMessage({message}) {
                const store = useModuleStore.getState()
                const SystemInitialized = useModuleStore.getState().portInfo
                 const newLog = useModuleStore.getState().AddLog;

                 newLog(message)

                switch (message.type) {
                    case "Registration":
                        store.registerModule(message.payload)
                        break
                    case "ModuleEvent":
                        store.dispatchModuleEvent(message.payload)
                        break
                    case "System":
                        if (SystemInitialized.status  == "connected"){
                            useModuleStore.getState().reset()
                        }
                        store.setSystemInfo(message.payload)
                        break
                }
            },
            PortStatus({status ,path}){
                useModuleStore.getState().setPort(path , status)
                if (status === "disconnected"){
                    useModuleStore.getState().reset()
                }
            }

        }

    }
})
export const electroview = new Electroview({ rpc });
