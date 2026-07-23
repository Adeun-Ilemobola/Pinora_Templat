import { PointSchema } from "@/lib/ModuleEven";
import { useEffect, useState } from "react";
import z from "zod";
import { Input } from "./ui/input";



type Point = z.infer<typeof PointSchema>;


type PointInputProp = {
    point: Point,
    Change: (newPoint: Partial<Point>) => void,
    disabled:boolean
    
}


export function PointInput({ point, Change , disabled =false }: PointInputProp) {
    const [draftX, setDraftX] = useState(`${point.x}`)
    const [draftY, setDraftY] = useState(`${point.y}`)

    useEffect(() => setDraftX(`${point.x}`), [point.x])
    useEffect(() => setDraftY(`${point.y}`), [point.y])

    function changeWithText(axis: keyof Point) {
        const draftXAxis = axis === "x" ? draftX : draftY;
        const toNumber = Number.parseInt(draftXAxis, 10);
        const clamped = Number.isNaN(toNumber)
            ? point[axis]
            : Math.min(Math.max(toNumber, -90), 90);

        Change({
            [axis]: clamped
        })
        if (axis === "x") {
            setDraftX(`${clamped}`)
        } else {
            setDraftY(`${clamped}`)
        }
    }

    function scrollUp(axis: keyof Point) {
        const clap = Math.min(Math.max(point[axis] + 1, -90), 90);
        Change({
            [axis]: clap
        })
        if (axis === "x") {
            setDraftX(`${clap}`)
        } else {
            setDraftY(`${clap}`)
        }

    }


    function scrollDowe(axis: keyof Point) {
        const clap = Math.min(Math.max(point[axis] - 1, -90), 90);
        Change({
            [axis]: clap
        })
        if (axis === "x") {
            setDraftX(`${clap}`)
        } else {
            setDraftY(`${clap}`)
        }


    }


    return (
        <div className="flex w-fit items-center gap-4 rounded-lg border border-border/60 bg-card/40 px-3 py-2 ">

            <div className=" flex items-center gap-2">
                <span className="text-sm font-medium text-muted-foreground">
                    X
                </span>
                <Input
                disabled={disabled}
                    max={90}
                    min={-90}
                    value={draftX}
                    className="w-14 text-center"
                    onBlur={() => {
                        changeWithText("x")

                    }}
                    onChange={(e) => {
                        setDraftX(e.target.value)
                    }}

                    onWheel={(event) => {
                        if (event.deltaY < 0) {
                            // console.log("Scroll up");
                            scrollUp("x")
                        } else if (event.deltaY > 0) {
                            // console.log("Scroll down");
                            scrollDowe("x")
                        }
                    }}
                />
            </div>

            <div className="h-5 w-px bg-border/60" />


            <div className=" flex items-center gap-2">
                <span className="text-sm font-medium text-muted-foreground">
                    Y
                </span>
                <Input
                 disabled={disabled}
                    max={90}
                    min={-90}
                    value={draftY}
                    className="w-14 text-center"
                    onBlur={() => {
                        changeWithText("y")

                    }}
                    onChange={(e) => {
                        setDraftY(e.target.value)
                    }}

                    onWheel={(event) => {
                        if (event.deltaY < 0) {
                            // console.log("Scroll up");
                            scrollUp("y")
                        } else if (event.deltaY > 0) {
                            // console.log("Scroll down");
                            scrollDowe("y")
                        }
                    }}
                />
            </div>





        </div>
    )




}
