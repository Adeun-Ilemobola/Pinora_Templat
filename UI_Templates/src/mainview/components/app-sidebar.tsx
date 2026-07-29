import {
    Collapsible,
    CollapsibleContent,
    CollapsibleTrigger,
} from "@/components/ui/collapsible"
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
    SidebarGroupLabel,
    SidebarGroupAction,



    SidebarMenuSub,
    SidebarMenuSubItem,
    SidebarMenuSubButton,
    SidebarMenuBadge,
} from "@/components/ui/sidebar"
import { useEffect, useMemo, useState } from "react"
import {


    Mail,
    Moon,
    Plus,
    Sun, Zap, X,
    ChevronRight,
    Cpu,
    Logs
} from "lucide-react"
import { Button } from "@/components/ui/button"
import { Separator } from "@/components/ui/separator"
import { useModuleStore } from "@runtime/ModuleStore"
import { Badge } from "@/components/ui/badge"



export function AppSidebar() {
    const [dark, setDark] = useState(true)
    const count = useModuleStore(
        (state) => Object.keys(state.modules).length
    )

    const modules = useModuleStore(
        (state) => state.modules
    )


    const data = useMemo(() => {
        const List = Object.values(modules)
        const StandAlone = List.filter(item => item.parent_id.length != 0)
            .map(item => ({

                id: item.id,
                lookUpId: item.lool_up_id,
                module_type: item.module_type,
                has_parent: item.parent_id.length > 10,
                parent_id: item.parent_id,


            }))


        const Grouping = List.filter(item => item.parent_id.length === 0)
            .map(item => ({

                id: item.id,
                lookUpId: item.lool_up_id,
                module_type: item.module_type,
                has_parent: item.parent_id.length > 10,
                parent_id: item.parent_id,


            }))

        return {
            StandAlone,
            Grouping

        }

    }, [modules])



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

                <Badge size={"lg"} variant={"outline"}>modules :{count}</Badge>
                <Separator />




            </SidebarHeader>
            <SidebarContent>
                <SidebarMenu>
                    <Collapsible className="group/standalone">
                        <SidebarMenuItem>
                            <CollapsibleTrigger
                                render={
                                    <SidebarMenuButton tooltip="Stand Alone Modules" />
                                }
                            >
                                <Cpu />
                                <span>Stand Alone Modules</span>

                                <ChevronRight
                                    className="
          ml-auto transition-transform
          group-data-[state=open]/standalone:rotate-90
        "
                                />
                            </CollapsibleTrigger>

                            <CollapsibleContent>
                                <SidebarMenuSub>
                                    {data.StandAlone.map((item) => (
                                        <SidebarMenuSubItem key={item.id}>
                                            <SidebarMenuSubButton>
                                                {item.lookUpId}
                                            </SidebarMenuSubButton>
                                        </SidebarMenuSubItem>
                                    ))}
                                </SidebarMenuSub>
                            </CollapsibleContent>
                        </SidebarMenuItem>
                    </Collapsible>


                    <Collapsible className="group/grouping">
                        <SidebarMenuItem>
                            <CollapsibleTrigger
                                render={
                                    <SidebarMenuButton tooltip="Grouping Modules" />
                                }
                            >
                                <Cpu />
                                <span>Grouping Modules</span>

                                <ChevronRight
                                    className="
          ml-auto transition-transform
          group-data-[state=open]/grouping:rotate-90
        "
                                />
                            </CollapsibleTrigger>

                            <CollapsibleContent>
                                <SidebarMenuSub>
                                    {data.Grouping.map((item) => (
                                        <SidebarMenuSubItem key={item.id}>
                                            <SidebarMenuSubButton>
                                                {item.lookUpId}
                                            </SidebarMenuSubButton>
                                        </SidebarMenuSubItem>
                                    ))}
                                </SidebarMenuSub>
                            </CollapsibleContent>
                        </SidebarMenuItem>
                    </Collapsible>
                </SidebarMenu>

                <SidebarMenu>
                    <SidebarMenuItem >
                        <SidebarMenuButton >
                            <Logs />

                            <span>Logs</span>

                        </SidebarMenuButton>
                    </SidebarMenuItem>
                </SidebarMenu>




            </SidebarContent>
            {/* <SidebarFooter /> */}
        </Sidebar>
    )
}
