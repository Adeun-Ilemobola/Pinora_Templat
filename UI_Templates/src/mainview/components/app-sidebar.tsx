import {
    Sidebar,
    SidebarContent,
    SidebarFooter,
    SidebarGroup,
    SidebarGroupContent,
    SidebarHeader,
    SidebarMenu,
    SidebarMenuButton,
    SidebarMenuItem,
} from "@/components/ui/sidebar"
import { useEffect, useState } from "react"
import {


    Mail,
    Moon,
    Plus,
    Sun, Zap, X
} from "lucide-react"
import { Button } from "./ui/button"
import { Separator } from "./ui/separator"
import { useModuleStore } from "../../Runtime/ModuleStore"



export function AppSidebar() {
    const [dark, setDark] = useState(true)
    const moduleCount = useModuleStore(
        (state) => Object.keys(state.modules).length,
    )


    useEffect(() => { document.documentElement.classList.toggle("dark", dark) }, [dark])
    return (
        <Sidebar>
            <SidebarHeader>

                <div className="flex flex-row items-center justify-center gap-2.5 p-1">
                    <div className="flex h-16 items-center gap-3 px-4">
                        <div className="grid size-9 place-items-center rounded-xl bg-primary text-primary-foreground">
                            <Zap className="size-5" /></div>
                        <div>
                            <div className="font-semibold">Pinora Studio</div>
                            <div className="text-xs text-muted-foreground">System console</div></div>
                    </div>
                    <Button size={"icon-lg"} variant={"ghost"} onClick={() => { setDark(pre => !pre) }}>
                        {dark ? <Sun /> : <Moon />}
                    </Button>
                </div>
                <Separator />




            </SidebarHeader>
            <SidebarContent>
                <SidebarGroup>
                    <SidebarGroupContent className="flex flex-col gap-2">

                        <SidebarMenu>

                            <SidebarMenuItem >
                                <SidebarMenuButton tooltip={"overview"}>
                                    <X />
                                    <span>{"Overview"}</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>

                            <SidebarMenuItem >
                                <SidebarMenuButton tooltip={"devices"}>
                                    <X />
                                    <span>{"Devices"}</span>
                                    <span>{moduleCount}</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>

                            <SidebarMenuItem >
                                <SidebarMenuButton tooltip={"logs"}>
                                    <X />
                                    <span>{"Logs"}</span>
                                </SidebarMenuButton>
                            </SidebarMenuItem>

                        </SidebarMenu>
                    </SidebarGroupContent>
                </SidebarGroup>

            </SidebarContent>
            {/* <SidebarFooter /> */}
        </Sidebar>
    )
}