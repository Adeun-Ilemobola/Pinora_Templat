import { createHashRouter } from "react-router";
import { RouterProvider } from "react-router/dom";
import { AppLayout } from "@/components/AppLayout";
import Dashboard from "@/page/Dashboard";
import "./index.css";
import LidarPage from "@/page/LidarPage";
import LogsPage from "@/page/LogsPage";
import { RouteError } from "@/components/RouteError";

// Hash routing works with Tauri's asset URLs without a server fallback.
// Each child route has its own ErrorBoundary so a rendering fault stays inside the
// layout; the root one is the last resort (PIN-TPL-006, Phase 1C).
const router = createHashRouter([{
  path: "/",
  Component: AppLayout,
  ErrorBoundary: RouteError,

  children: [{
    index: true,
    Component: Dashboard,
    ErrorBoundary: RouteError
  }, 
  {
    path: "lidar",
    Component: LidarPage,
    ErrorBoundary: RouteError
  },
  {
    path: "logs",
    Component: LogsPage,
    ErrorBoundary: RouteError
  }]
}]);

export default function App() {
  return <RouterProvider router={router} />;
}
