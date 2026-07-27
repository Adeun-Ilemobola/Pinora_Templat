


import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge';
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator';
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { PortConnectionScheme, useListenStore } from '@/lib/ListenStore';
import {
  CircuitryIcon,
  HardDriveIcon,
  MemoryIcon,
} from '@phosphor-icons/react';

import { useEffect, useState } from 'react'

function formatDuration(ms: number) {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000))
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  const pad = (n: number) => String(n).padStart(2, "0")
  return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`
}

export default function PortSettings() {
  const portInfo = useListenStore((state) => state.portInfo)
  const connect = useListenStore((state) => state.connect)
  const status = useListenStore((state) => state.status)
  const error = useListenStore((state) => state.error)
  const systemInfo = useListenStore((state) => state.SystemInfo)
  const getPorts = useListenStore((state) => state.getPorts)
  const setPortInfo = useListenStore((state) => state.setPortInfo)
  const startConnectionTime = useListenStore((state) => state.commitTime)

  const [elapsed, setElapsed] = useState(0)
  const [ports, setPort] = useState<string[]>([])

  useEffect(() => {
    const loadPorts = async () => {
      const data = await getPorts()
      setPort(data)

    }

    loadPorts()

  }, [])

  useEffect(() => {
    if (!startConnectionTime) {
      setElapsed(0)
      return
    }

    const update = () => {
      setElapsed(Date.now() - startConnectionTime.getTime())
    }

    update()
    const id = setInterval(update, 1000)
    return () => clearInterval(id)
  }, [startConnectionTime])
  function MakeConnect() {
    const val = PortConnectionScheme.safeParse(portInfo)
    if (val.success) {
      connect(
        val.data.port,
        val.data.baudRate
      )

    }

  }

  return (
    <div className=' min-h-screen  w-full flex flex-col gap-0.5 p-3 items-center'>
      <h1 className=' text-4xl'>PortSettings</h1>


      <div className=' flex  flex-col gap-2'>

        <div className=' flex flex-row gap-4'>


          <div className=' flex flex-col gap-1'>
            <Label>
              Ports
            </Label>

            <Select
              value={portInfo.port}
              onValueChange={((val) => {
                if (val === "__none__") {
                  return
                }
                setPortInfo({ port: val })
              })}
            >
              <SelectTrigger className="w-45">
                <SelectValue placeholder="ports" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  {ports.length === 0 ? (
                    <SelectItem value="__none__" disabled>No ports available</SelectItem>
                  ) : (
                    ports.map(port => (
                      <SelectItem key={port} value={port}>{port}</SelectItem>
                    ))
                  )}
                </SelectGroup>
              </SelectContent>
            </Select>

          </div>

          <div className=' flex flex-col gap-1'>
            <Label>
              baud Rate
            </Label>
            <Input
              value={portInfo.baudRate}
              onChange={(e) => {
                const val = e.target.value
                const toNum = Number(val)
                if (!isNaN(toNum)) {
                  setPortInfo({ baudRate: toNum })
                }
              }}

            />
          </div>

        </div>


        <div className=' flex flex-col gap-2'>
          <Button
            onClick={() => {
              MakeConnect()
            }}
          >
            Connect to port
          </Button>
          <p>
            {status}
          </p>

          {error && (<>
            <p className=' text-amber-700'>
              {error}

            </p>

          </>)}

        </div>

      </div>

      {startConnectionTime && (
        <div className=' flex flex-col items-center gap-1 mt-2'>
          <Label>Connected for</Label>
          <p className=' text-2xl font-mono tabular-nums'>
            {formatDuration(elapsed)}
          </p>
        </div>
      )}


      {systemInfo && (
        <Card className="mt-5 w-full max-w-2xl" size="sm">
          <CardHeader>
            <div className="flex items-center gap-3">
              <div className="grid size-10 place-items-center rounded-lg bg-primary/10 text-primary">
                <CircuitryIcon className="size-5" weight="duotone" />
              </div>
              <div>
                <CardTitle>System information</CardTitle>
                <CardDescription>Runtime details reported by the connected device.</CardDescription>
              </div>
            </div>
            <CardAction>
              <Badge variant="secondary">ESP-IDF {systemInfo.esp_idf_version}</Badge>
            </CardAction>
          </CardHeader>

          <CardContent className="space-y-4">
            <section>
              <div className="mb-2 flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                <MemoryIcon className="size-4" weight="duotone" />
                Memory
              </div>
              <dl className="grid overflow-hidden rounded-lg border sm:grid-cols-2">
                <SystemInfoItem label="Total heap" value={systemInfo.total_heap} />
                <SystemInfoItem label="Currently free" value={systemInfo.current_free_heap} />
                <SystemInfoItem label="Lowest free" value={systemInfo.lowest_free_heap} />
                <SystemInfoItem label="Largest allocation" value={systemInfo.largest_allocation} />
              </dl>
            </section>

            <Separator />

            <section>
              <div className="mb-2 flex items-center gap-2 text-xs font-medium uppercase tracking-wider text-muted-foreground">
                <HardDriveIcon className="size-4" weight="duotone" />
                Storage
              </div>
              <dl className="grid overflow-hidden rounded-lg border sm:grid-cols-2">
                <SystemInfoItem label="Flash capacity" value={systemInfo.flash} />
                <SystemInfoItem label="Maximum app slot" value={systemInfo.maximum_app_slot} />
              </dl>
            </section>
          </CardContent>
        </Card>
      )}

    </div>
  )
}

function SystemInfoItem({ label, value }: { label: string; value: string }) {
  return (
    <div className="border-b p-3 last:border-b-0 sm:border-r sm:nth-[2n]:border-r-0 sm:nth-last-[-n+2]:border-b-0">
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd className="mt-1 break-words font-mono text-sm font-medium">{value}</dd>
    </div>
  )
}
