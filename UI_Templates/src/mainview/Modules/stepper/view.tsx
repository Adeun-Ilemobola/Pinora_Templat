import { Input } from '@/components/ui/input'
import { Slider } from '@/components/ui/slider'
import { StepperMotorDefinition } from './definition';
import { memo, useMemo, useState } from 'react';
import z from 'zod';
import ModuleCore from "@/components/ModuleCore";
import type { Commandtype as Command } from "@shared/Protocol/ModuleCommand";
import { ArrowBigRight } from 'lucide-react';
import { Badge } from '@/components/ui/badge';


type HoverPoint = {
    x: number,
    y: number,
    zoneX: number,
    zoneY: number,
}

type StepperCardProps = {
    module: StepperMotorDefinition;
    sendCommand: (command: Command) => Promise<void>;
    Disable: boolean
}
const stepsPerRevolution = 4096;

export const StepperCard = memo(({ module, sendCommand, Disable }: StepperCardProps) => {
    const [step, setStep] = useState("0")
    const [degrees, setdegrees] = useState("0")
    const [result, setResult] = useState({
        stepValue: 0.0,
        degreesPer: 0.0
    })
    const [hoverPoint, UpdateHoverPoint] = useState<HoverPoint | null>(null)
    const  [angleTarget , setAngleTarget] = useState<number>(0.0)



    function angleToSteps() {
        console.log("~oooo~")
        // Convert the requested angle into a step count
        const toNumber = Number.parseFloat(degrees)
        const isNull = Number.isNaN(toNumber)
        console.log(`toNumber :${toNumber} | isNull:${isNull}`)
        if (isNull) { return }
        const b = stepsPerRevolution / 360;
        setResult(pre => ({
            ...pre,
            stepValue: b * toNumber
        }))
    }
    function degreesPerStep() {
        // Your calculation here
        const toNumber = Number.parseFloat(step)
        const isNull = Number.isNaN(toNumber)
        if (isNull) { return }
        const b = 360 / stepsPerRevolution;

        setResult(pre => ({
            ...pre,
            degreesPer: (b * toNumber)
        }))
    }
    function PointerMove(ev: React.PointerEvent<HTMLDivElement>) {
        const block = ev.currentTarget;
        const bounds = block.getBoundingClientRect();
        const pointX = (ev.clientX - bounds.left)
        const pointY = (ev.clientY - bounds.top);

        UpdateHoverPoint({
            x: pointX,
            y: pointY,
            zoneX: bounds.width,
            zoneY: bounds.height
        })

    }

    const root_angle = useMemo(() => {
        if (!hoverPoint) {
            return 0.0
        }
        const radians = Math.atan2((hoverPoint?.y - hoverPoint.zoneY / 2), (hoverPoint.x - hoverPoint.zoneX / 2));
        const degrees = radians * (360 / Math.PI);

        return degrees
    }, [hoverPoint])

    function UpdateAngleTarget(ev: React.ChangeEvent<HTMLInputElement>) {
        if (module.state.mode !== "Idle") {
            return
        }
        const value = Number.parseFloat(ev.target.value)
        if (Number.isNaN(value)) {
            return
        }
        setAngleTarget(value)
    }

     function UpdateBoxAngleTarget() {
         if (module.state.mode !== "Idle") {
            return
        }
        if (!hoverPoint) {
            return
        }
       
        sendCommand({
            module_type:"StepperMotor",
            id:module.id,
            payload:{
                command:"MoveToAngle",
                angle:Number.parseFloat(angleTarget.toFixed(2))
            }

        })
    }
    

    function PointerLeave(ev: React.PointerEvent<HTMLDivElement>) {
        UpdateHoverPoint(null)

    }





    return (
        <ModuleCore
            id={module.id}
            manuel_id={module.lool_up_id}
            moduletype={module.module_type}
        >
            <section className="flex flex-col gap-4 items-center">

                <Badge>
                    {module.state.mode}
                </Badge>

                <div className="flex flex-row gap-2 items-center">   

                     <Input  value={module.state.angle} readOnly />
                     <div>
                        <ArrowBigRight />
                     </div>
                    <Input  value={angleTarget.toFixed(2)} readOnly />

                </div>

                <div
                    className="h-70 w-70 relative ring ring-fuchsia-600 rounded-full"
                    onPointerMove={PointerMove}
                    onPointerLeave={PointerLeave}
                    onClick={()=>{
                       setAngleTarget(root_angle)
                       UpdateBoxAngleTarget()
                    }}
                >
                    {/* {hoverPoint && (
					<div
						className="ring ring-green-800/70 p-2 absolute pointer-events-none whitespace-nowrap transform translate-x-1/10 -translate-y-1/5"
						style={{
							top: `${hoverPoint.y}px`,
							left: `${hoverPoint.x}px`
						}}
					>
						({Math.round(hoverPoint.x - hoverPoint.zoneX / 2)}, {Math.round(hoverPoint.y - hoverPoint.zoneY / 2)})
					</div>
				)} */}

                    <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 ring ring-red-600 rounded-md p-3 text-xl">
                        {root_angle.toFixed(2)}°
                    </div>
                </div>


            </section>


        </ModuleCore>
    )
})