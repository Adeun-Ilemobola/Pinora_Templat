
import { Copy } from 'lucide-react';
import { Badge } from '../ui/badge';
import { Button } from '../ui/button';
import { Card, CardContent, CardHeader } from '../ui/card';
import { Separator } from '../ui/separator';
import { toast } from 'sonner';


type ModuleCoreProp = {
    children: React.ReactNode
    id: string,
    manuel_id: string,
    moduletype: string
}
export default function ModuleCore({ children, id, manuel_id, moduletype }: ModuleCoreProp) {
    async function copyMid() {
        try {
            await navigator.clipboard.writeText(manuel_id);
            toast.success(`Text copied to clipboard!`)
        } catch (err) {
            console.error('Failed to copy: ', err);
        }

    }
    return (
        <Card >
            <CardHeader className='flex flex-row gap-6' >
                <div className=' flex flex-col gap-2.5'>
                    <div className=' flex flex-row items-center gap-5'>
                        <p className=' text-xs text-accent-foreground/35'>{id}</p>
                        <Badge size={"lg"}  variant={"destructive"}>
                            {moduletype}
                        </Badge>
                    </div>



                    <Button className=' w-fit' variant={"outline"} onClick={copyMid}>
                        <Copy />{manuel_id}
                    </Button>
                </div>




            </CardHeader>
            <Separator />

            <CardContent className='flex'>

                {children}

            </CardContent>
        </Card>
    )
}
