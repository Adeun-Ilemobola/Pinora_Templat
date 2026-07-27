import { StrictMode } from "react";
import "./index.css";

import App from "./App";
import { TooltipProvider } from "./components/ui/tooltip";
import { Toaster } from "sonner";
import {
  createBrowserRouter,
  RouterProvider,
} from "react-router-dom";
import { createRoot } from "react-dom/client";
import Layout from "./lib/Layout";

const router = createBrowserRouter([
  {
    element: <Layout />,
    children: [
      {
        index: true,
        element: <App />,
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