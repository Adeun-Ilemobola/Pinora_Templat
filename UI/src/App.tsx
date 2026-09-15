import { createHashRouter } from "react-router";
import { RouterProvider } from "react-router/dom";
import { AppLayout } from "@/components/AppLayout";
import Dashboard from "@/page/Dashboard";
import "./index.css";
import LidarPage from "@/page/LidarPage";
import LogsPage from "@/page/LogsPage";

// Hash routing works with Tauri's asset URLs without a server fallback.
const router = createHashRouter([{
  path: "/",
  Component: AppLayout,

  children: [{
    index: true,
    Component: Dashboard
  }, {
    path: "lidar",
    Component: LidarPage
  }, {
    path: "logs",
    Component: LogsPage
  }]
}]);

export default function App() {
  return <RouterProvider router={router} />;
}
