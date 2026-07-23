import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/page/App";
import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Layout } from "./Layout";
import Dashboard from "./page/Dashboard";
import Devices from "./page/Devices";
import PortSettings from "./page/PortSettings";
import Logs from "./page/Logs";
import NotFound from "./page/NotFound";

import { warn, debug, trace, info, error } from '@tauri-apps/plugin-log';

function forwardConsole(
  fnName: 'log' | 'debug' | 'info' | 'warn' | 'error',
  logger: (message: string) => Promise<void>
) {
  const original = console[fnName];
  console[fnName] = (message) => {
    original(message);
    logger(message);
  };
}

forwardConsole('log', trace);
forwardConsole('debug', debug);
forwardConsole('info', info);
forwardConsole('warn', warn);
forwardConsole('error', error);

const router = createBrowserRouter([
  {
    path: "/",
    element: <Layout />,
    children: [
      { index: true, element: <App /> },
      { path: "Dashboard", element: <Dashboard /> },
      { path: "Devices", element: <Devices /> },
      { path: "PortSettings", element: <PortSettings /> },
      { path: "Logs", element: <Logs /> }

    ],
    
  },
  { path: "*", element: <NotFound /> } 
]);
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <RouterProvider router={router} />
  </React.StrictMode>,
);
