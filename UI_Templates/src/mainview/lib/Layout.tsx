import { AppSidebar } from '@/components/app-sidebar';
import { Button } from '@/components/ui/button';
import {
    SidebarProvider,
    SidebarTrigger,
} from '@/components/ui/sidebar';
import {
    Outlet,
    useLocation,
    useNavigate,
} from 'react-router-dom';

export default function Layout() {
    const location = useLocation();
    const navigate = useNavigate();

    const isHomePage = location.pathname === '/';

    const pageName = location.pathname
        .split('/')
        .filter(Boolean)
        .map((segment) => decodeURIComponent(segment))
        .join(' / ');

    return (
        <SidebarProvider>
            <AppSidebar />

            <main className="relative flex h-screen w-full flex-col overflow-y-auto">
                <header className="sticky top-0 z-20 flex h-14 shrink-0 items-center gap-1 border-b bg-background/85 px-2 backdrop-blur-xl md:px-4">
                    <SidebarTrigger size="icon-lg" />

                    {!isHomePage && (
                        <>
                            <span className="text-sm font-medium text-muted-foreground">
                                {pageName}
                            </span>

                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                className="ml-auto"
                                onClick={() => navigate(-1)}
                            >
                                Back
                            </Button>
                        </>
                    )}
                </header>

                <section className="w-full flex-1 p-2">
                    <Outlet />
                </section>
            </main>
        </SidebarProvider>
    );
}