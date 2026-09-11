import { useState } from "react";
import "./index.css";
import { TransporForm } from "./components/TransporForm";
import { useModuleFront } from "./lib/Modulefront";
import { LedView } from "./lib/Modules/Led";

function App() {
  const stat = useModuleFront(store =>store.PortStat);
  return (
    <main className="bg-background text-foreground min-h-screen flex flex-col ">
      <h1>Welcome to Tauri + React</h1>

      <TransporForm />

      {stat === "Connected" && <>
        <LedView
          id="led1"
        />
        <LedView
          id="led2"
        />
        <LedView
          id="led3"
        />
      </>}


      
      
    </main>
  );
}

export default App;
