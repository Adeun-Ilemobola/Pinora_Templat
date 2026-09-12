import { NavLink, Outlet, useMatch } from "react-router";
import { CircuitBoard, LayoutDashboard, Moon, ScanEye, Sun } from "lucide-react";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { Separator } from "@/components/ui/separator";
import { Button } from "@/components/ui/button";
import { useTheme } from "@/components/theme-provider";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useModuleFront } from "@/lib/Modulefront";

/** Persistent desktop shell; route content owns its own hardware subscriptions. */
export function AppLayout() {
  const active = !!useMatch("/");
  const { theme, setTheme } = useTheme();
  return (
    <TooltipProvider>
      <SidebarProvider>
        <Sidebar collapsible="icon">
          <SidebarHeader className="border-b border-sidebar-border py-5">
            <SidebarMenu>
              <SidebarMenuItem>
                <SidebarMenuButton size="lg" render={<NavLink to="/" />}>
                  <CircuitBoard className="size-5" />
                  <span className="text-lg font-semibold tracking-tight">
                    Pinora
                  </span>
                </SidebarMenuButton>
              </SidebarMenuItem>
            </SidebarMenu>
          </SidebarHeader>
          <SidebarContent>
            <SidebarGroup>
              <SidebarMenu>
                <SidebarMenuItem className=" gap-2">

                  <SidebarMenuButton
                    isActive={active}
                    tooltip="Lidar"
                    render={<NavLink to="/lidar" end />}
                  >
                    <ScanEye />
                    <span>Lidar</span>
                  </SidebarMenuButton>


                  <SidebarMenuButton
                    isActive={active}
                    tooltip="Dashboard"
                    render={<NavLink to="/" end />}
                  >
                    <LayoutDashboard />
                    <span>Dashboard</span>
                  </SidebarMenuButton>


                </SidebarMenuItem>
              </SidebarMenu>
            </SidebarGroup>
          </SidebarContent>
        </Sidebar>
        <SidebarInset className="min-w-0">
          <header className="flex h-16 shrink-0 items-center gap-3 border-b px-4 md:px-6">
            <SidebarTrigger />
            <Separator orientation="vertical" className="h-4" />
            <span className="text-sm text-muted-foreground">
              Control workspace
            </span>
            <Button
              className="ml-auto"
              variant="ghost"
              size="icon"
              aria-label={
                theme === "dark"
                  ? "Switch to light theme"
                  : "Switch to dark theme"
              }
              onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
            >
              {theme === "dark" ? <Sun /> : <Moon />}
            </Button>
          </header>
          <div className="mx-auto w-full max-w-7xl space-y-6 p-4 md:p-6 lg:p-8">
            <Outlet />
          </div>
        </SidebarInset>
      </SidebarProvider>
    </TooltipProvider>
  );
}
