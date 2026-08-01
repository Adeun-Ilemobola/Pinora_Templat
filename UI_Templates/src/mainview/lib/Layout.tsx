
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from '@/components/app-sidebar';
import { Outlet } from "react-router-dom";
import {
 useLocation, 
} from "react-router-dom";
import { Button } from "@/components/ui/button";
interface LayoutProps {
    children: React.ReactNode 
}

const Layout = () => {
  const location = useLocation();              
    return (
    <SidebarProvider>
      <AppSidebar />
      <main className='flex flex-col h-screen w-full overflow-hidden overflow-y-scroll relative'>
        <header className='sticky top-0 z-20 flex h-14 items-center gap-1 border-b bg-background/85 px-2 backdrop-blur-xl md:px-4  flex-row'>
        <SidebarTrigger size={"icon-lg"} /> 
        {location.pathname !== "/" && (<>
        <span className="text-sm font-medium text-muted-foreground">
            {location.pathname}
        </span>
        
         <Button variant={"outline"} size={"sm"} className="ml-auto" onClick={()=>{
            window.history.back()
        }}>
            Back
        </Button>
        </>)}
       
        
  
        </header>
        <section className='w-full h-full flex-1 p-2 '>
             <Outlet />
        </section>
       
      </main>
    </SidebarProvider>
    );
}

export default Layout;
