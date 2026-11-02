import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ThemeProvider } from "@/components/theme-provider"

import { warn, error } from '@tauri-apps/plugin-log';

// Only warnings and errors are forwarded to the Tauri log (audit Phase 2D,
// PIN-TPL-011): forwarding every console call cost one IPC round trip per message.

function forwardConsole(
  fnName: 'log' | 'debug' | 'info' | 'warn' | 'error',
  logger: (message: string) => Promise<void>
) {
  const original = console[fnName];

  console[fnName] = (...args: unknown[]) => {
    original(...args);

    const message = args
      .map(arg => {
        if (typeof arg === "string") {
          return arg;
        }

        try {
          return JSON.stringify(arg);
        } catch {
          return String(arg);
        }
      })
      .join(" ");

    logger(message).catch(() => {
      // The log plugin is unavailable (for example in a browser preview).
    });
  };
}

forwardConsole('warn', warn);
forwardConsole('error', error);
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <App />
    </ThemeProvider>
  </React.StrictMode>,
);
