import { StrictMode } from "react";
import "./index.css";

import App from "./App";
import { TooltipProvider } from "./components/ui/tooltip";
import { Toaster } from "sonner";
import {
  createHashRouter,
  RouterProvider,
} from "react-router-dom";
import { createRoot } from "react-dom/client";
import Layout from "./lib/Layout";
import { LogsBox } from "./Pages/logs";

const router = createHashRouter([
  {
    element: <Layout />,
    children: [
      {
        index: true,
        element: <App />,
      },
      {
        path: "logs",
        element: <LogsBox />,
      },
    ],
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <TooltipProvider>
      <RouterProvider router={router} />
      <Toaster richColors position="bottom-right" />
    </TooltipProvider>
  </StrictMode>,
);