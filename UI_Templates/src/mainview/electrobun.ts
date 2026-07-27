// src/mainview/electrobun.ts

import { Electroview } from "electrobun/view";
import type { AppRPC } from "../shared/rpc";
import { useModuleStore } from "../Runtime/ModuleStore";


const rpc = Electroview.defineRPC<AppRPC>({
    handlers: {
        requests: {



        },
        messages: {
            incomingMessage({message}) {
                const store = useModuleStore.getState()

                switch (message.type) {
                    case "Registration":
                        store.registerModule(message.payload)
                        break
                    case "ModuleEvent":
                        store.dispatchModuleEvent(message.payload)
                        break
                    case "System":
                        store.setSystemInfo(message.payload)
                        break
                }
            },

        }

    }
})
export const electroview = new Electroview({ rpc });