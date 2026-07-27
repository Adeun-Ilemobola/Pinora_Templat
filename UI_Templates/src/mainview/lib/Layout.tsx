
import type { FC } from 'react';
import { SidebarProvider, SidebarTrigger } from "@/components/ui/sidebar"
import { AppSidebar } from '@/components/app-sidebar';

interface LayoutProps {
    children: React.ReactNode
}

const Layout: FC<LayoutProps> = ({children}) => {
    return (
    <SidebarProvider>
      <AppSidebar />
      <main className='flex flex-col h-screen w-full overflow-hidden overflow-y-scroll relative'>
        <header className='sticky top-0 z-20 flex h-22 items-center gap-3 border-b bg-background/85 px-2 backdrop-blur-xl md:px-4'>
        <SidebarTrigger size={"lg"} />
        </header>
        <section className='w-full h-full flex-1 p-2 bg-emerald-900/55'>
             {children}
        </section>
       
      </main>
    </SidebarProvider>
    );
}

export default Layout;
