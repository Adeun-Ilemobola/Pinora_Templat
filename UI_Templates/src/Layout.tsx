import { Outlet } from "react-router-dom";
import { useNavigate } from "react-router-dom";

import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupContent,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarInset,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { TooltipProvider } from "@/components/ui/tooltip"
import { ThemeProvider } from "@/components/theme-provider"
import { useListenStore } from "./lib/ListenStore";
import { useModuleStore } from "./lib/ModuleStore";
import { useEffect } from "react";
import { Toaster } from "@/components/ui/sonner"

const navMain = [
  { label: "Home", icon: "⊞", path: "/" },
  { label: "Dashboard", icon: "⊞", path: "/Dashboard" },
  { label: "Devices", icon: "⚡", badge: "3", path: "/Devices" },
  // { label: "Flash Firmware", icon: "↑" },
]

const navSettings = [
  { label: "Port Settings", icon: "⚙", path: "/PortSettings" },
  { label: "Logs", icon: "☰", path: "/Logs" },
]

export function Layout() {
  const  startListeners = useListenStore((state) => state.startListeners)
  const  stopListeners = useListenStore((state) => state.stopListeners)
  const portInfo = useListenStore((state) => state.portInfo)

  useEffect(() => {
    if (!portInfo.port) {
      stopListeners()
      return
    }
    startListeners()

    return () => {
      stopListeners()
    }
  }, [portInfo.port])

  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <Toaster 
        richColors
        position="bottom-center"
      />
      <TooltipProvider>
        <SidebarProvider>
          <AppSidebar />
          <SidebarInset className=" h-svh">
            <header className="flex h-12 items-center gap-2 border-b px-4">
              <SidebarTrigger />
              <span className="text-sm font-medium">ESP Tool</span>
            </header>
            <div className="flex-1 min-h-0 overflow-y-auto">   {/* the one real scroll container */}
              <Outlet />
            </div>
          </SidebarInset>
        </SidebarProvider>
      </TooltipProvider>
    </ThemeProvider>
  );
}

function AppSidebar() {
  const navigate = useNavigate();
  const moduleCount = useModuleStore((state) => state.ModuleCount())

  return (
    <Sidebar>
      <SidebarHeader className="px-4 py-3 font-semibold text-sm">
        ESP Tool
      </SidebarHeader>

      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupLabel>Main</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navMain.map((item) => (
                <SidebarMenuItem
                  key={item.label}
                  onClick={() => {
                    navigate(item.path)
                  }}
                >
                  <SidebarMenuButton>
                    <span>{item.icon}</span>
                    <span>{item.label}</span>
                  </SidebarMenuButton>
                  {item.badge && <SidebarMenuBadge>{moduleCount}</SidebarMenuBadge>}
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>

        <SidebarGroup>
          <SidebarGroupLabel>Settings</SidebarGroupLabel>
          <SidebarGroupContent>
            <SidebarMenu>
              {navSettings.map((item) => (
                <SidebarMenuItem
                  key={item.label}
                  onClick={() => {
                    navigate(item.path)
                  }}
                >
                  <SidebarMenuButton>
                    <span>{item.icon}</span>
                    <span>{item.label}</span>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter className="px-4 py-3 text-xs text-muted-foreground">
        v0.1.0
      </SidebarFooter>
    </Sidebar>
  )
}
