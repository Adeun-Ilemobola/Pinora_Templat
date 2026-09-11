import { createHashRouter } from "react-router";
import { RouterProvider } from "react-router/dom";
import { AppLayout } from "@/components/AppLayout";
import Dashboard from "@/page/Dashboard";
import "./index.css";

// Hash routing works with Tauri's asset URLs without a server fallback.
const router = createHashRouter([
  { path: "/", Component: AppLayout, children: [{ index: true, Component: Dashboard }] },
]);

export default function App() {
  return <RouterProvider router={router} />;
}
