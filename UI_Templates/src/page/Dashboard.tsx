import Grid from "@/components/Grid"
import { ServoCard } from "@/components/Modules/Servo"
import { PointInput } from "@/components/PointInput"
import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader } from "@/components/ui/card"
import { PointSchema } from "@/lib/ModuleEven"
import { useModuleStore } from "@/lib/ModuleStore"
import { CubeIcon } from "@phosphor-icons/react"
import { useEffect, useState } from "react"
import z from "zod"


type Point = z.infer<typeof PointSchema>;


export default function Dashboard() {
  const lidar = useModuleStore((state) => selectModule(state, "lidar", "Lidar"))
  const servoX = useModuleStore((state) => selectModule(state, "servo_x", "Servo"))
  const servoY = useModuleStore((state) => selectModule(state, "servo_y", "Servo"))
  const sendCommand = useModuleStore((state) => state.sendCommand)
  const [pointMax, setPointMax] = useState<Point>(() =>
    lidar?.state.ROI.max ?? { x: 0, y: 0 },
  )
  const [pointMin, setPointMin] = useState<Point>(() =>
    lidar?.state.ROI.min ?? { x: 0, y: 0 },
  )

  useEffect(() => {
    if (!lidar) return
    setPointMax(lidar.state.ROI.max)
    setPointMin(lidar.state.ROI.min)
  }, [
    lidar?.state.ROI.max.x,
    lidar?.state.ROI.max.y,
    lidar?.state.ROI.min.x,
    lidar?.state.ROI.min.y,
  ])

  if (!lidar || !servoX || !servoY) {
    return (
      <div className="mx-auto flex min-h-[calc(100svh-3rem)] w-full max-w-3xl items-center justify-center p-6">
        <Card className="w-full max-w-md text-center" size="sm">
          <CardContent className="flex flex-col items-center py-10">
            <div className="mb-4 grid size-12 place-items-center rounded-full bg-primary/10 text-primary">
              <CubeIcon className="size-6" weight="duotone" />
            </div>
            <h1 className="font-heading text-2xl font-medium">Waiting for a LiDAR</h1>
            <p className="mt-2 max-w-sm text-sm leading-6 text-muted-foreground">
              Connect a LiDAR module to view its identity, scan area, and axis controls.
            </p>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-2 h-full min-h-0 w-full p-1.5">
      <h1 className=' text-4xl'>Dashboard</h1>

      <div className=" flex flex-row gap-6 items-center p-4">
        <Button
          disabled={lidar.state.state === "Scanning"}
          variant={(lidar.state.state === "Scanning") ? "destructive" : "default"}
          onClick={() => {
            sendCommand({
              id: lidar.id,
              module_type: "Lidar",
              payload: {
                command: "StartScan"
              }
            })
          }}
        >
          Start Scan
        </Button>
        <Button
          disabled={lidar.state.state === "StopScan" || lidar.state.state === "Idol"}
          variant={(lidar.state.state === "StopScan" || lidar.state.state === "Idol") ? "destructive" : "default"}
          onClick={() => {
            sendCommand({
              id: lidar.id,
              module_type: "Lidar",
              payload: {
                command: "StopScan"
              }
            })
          }}
        >
          Stop Scan
        </Button>
      </div>

      <Card className="shrink-0">
        <CardHeader>
          RoI
        </CardHeader>
        <CardContent className="flex flex-wrap items-center gap-4">

          <div className=" flex flex-col w-fit justify-center items-center">
            <h2>
              Max Pont
            </h2>
            <PointInput
              disabled={lidar.state.state === "Scanning"}
              point={pointMax}
              Change={(v) => {
                setPointMax(pre => ({ ...pre, ...v }))
              }}
            />
          </div>


          <div className=" flex flex-col w-fit justify-center items-center">
            <h2>
              Min
            </h2>
            <PointInput
              disabled={lidar.state.state === "Scanning"}
              point={pointMin}
              Change={(v) => {
                setPointMin(pre => ({ ...pre, ...v }))
              }}
            />
          </div>
        </CardContent>
      </Card>


      <div className=" flex  flex-row gap-3.5  items-center p-1">
        <ServoCard Disable={lidar.state.state === "Scanning"} module={servoX} sendCommand={sendCommand} />
        <ServoCard Disable={lidar.state.state === "Scanning"} module={servoY} sendCommand={sendCommand} />
      </div>

      <Grid
        move_point={(p) => {
          sendCommand({
            id: lidar.id,
            module_type: "Lidar",
            payload: {
              command: "MovePos",
              p,
            }
          })

        }}
        max={pointMax}
        min={pointMin}
        map={lidar.state.map}
        setRoi={(nextMin, nextMax) => {
          setPointMin(nextMin)
          setPointMax(nextMax)
          sendCommand({
            id: lidar.id,
            module_type: "Lidar",
            payload: {
              command: "Roi",
              min: nextMin,
              max: nextMax,
            }
          })

        }}

      />

    </div>
  )
}

function selectModule<T extends "Lidar" | "Servo">(
  state: ReturnType<typeof useModuleStore.getState>,
  lookupId: string,
  moduleType: T,
) {
  const id = state.LookUp_ID_refTo_ID[lookupId]
  const module = id ? state.modules[id] : undefined
  return module?.module_type === moduleType
    ? module as Extract<typeof module, { module_type: T }>
    : undefined
}
