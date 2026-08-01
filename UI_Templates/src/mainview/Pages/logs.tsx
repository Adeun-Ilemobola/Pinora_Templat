import { Separator } from '@/components/ui/separator';
import { useVirtualizer } from '@tanstack/react-virtual';

import { InComingMessageSchemaType, moduleTypeIdentifier } from '@shared/Protocol/ModuleDefinitionSchema';
import React, { useRef, useState, useMemo } from 'react';
import z from 'zod';
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"

type LogsProps = {}
type LogFromType = "Event" | "log" | null

const items: { label: string, value: LogFromType }[] = [
    { label: "Event", value: "Event" },
    { label: "log", value: "log" },
    { label: "None", value: null },
]

const zLogFrom_Event = z.object({
    Type: z.literal("Event"),
    eventType: InComingMessageSchemaType.nullable(),
    dentifier: moduleTypeIdentifier.nullable()

})
const log_priority = z.enum(["Low", "Medium", "High", "Critical"])
const zLogFrom_Log = z.object({
    Type: z.literal("log"),
    priority: log_priority.nullable(),

})
const zLogFrom = z.discriminatedUnion("Type", [
    zLogFrom_Event, zLogFrom_Log
])

type zLogFromType = z.infer<typeof zLogFrom>

export const LogsBox = ({ }: LogsProps) => {
    const parentRef = useRef(null)

    const [searchMode, setSearchMode] = useState<zLogFromType | null>(null);


    const logList = useMemo(() => {
        if (!searchMode) {
            return
        }
        switch (searchMode.Type) {
            case "Event": {
                break;
            }
            case "log": {
                break;
            }
        }

    }, [searchMode]);

    const rowVirtualizer = useVirtualizer({
        count: 10000,
        getScrollElement: () => parentRef.current,
        estimateSize: () => 35,
    })
    function onSearchModeChange(mode: LogFromType) {
        switch (mode) {
            case "Event": {
                setSearchMode({
                    Type: "Event",
                    eventType: null,
                    dentifier: null
                })
                break;
            }
            case "log": {
                setSearchMode({
                    Type: "log",
                    priority: null
                })
                break;
            }

            default:
                setSearchMode(null)
                break;
        }

    }

    const LoadMode = useMemo(() => {
        if (!searchMode) {
            return (<></>)
        }
        switch (searchMode.Type) {
            case "Event": {

                return (
                    <EventSearchMode
                        data={searchMode}
                        update={(v) => {
                            setSearchMode((prev) => {
                                if (!prev) {
                                    return prev
                                }
                                return {
                                    ...prev,
                                    ...v
                                }
                            })


                        }}
                    />)

            }
            case "log": {
                return (
                    <LogSearchMode
                        data={searchMode}
                        update={(v) => {
                            setSearchMode((prev) => {
                                if (!prev) {
                                    return prev
                                }
                                return {
                                    ...prev,
                                    ...v
                                }
                            })

                        }}
                    />)
            }
        }


    }, [searchMode])


    return (
        <main className=' flex flex-1 flex-col w-full'>
            <header className=' sticky top-13 h-13 z-19 flex flex-row gap-2 p-2 items-center justify-end'>

                {LoadMode}
                <Separator orientation="vertical" />

                <Select
                    items={items}
                    onValueChange={(v) => {
                        onSearchModeChange(v as LogFromType)
                    }}
                >
                    <SelectTrigger className="w-50">
                        <SelectValue placeholder="Mode" />
                    </SelectTrigger>
                    <SelectContent>
                        <SelectGroup>
                            {items.map((item) => (
                                <SelectItem key={item.value} value={item.value}>
                                    {item.label}
                                </SelectItem>
                            ))}
                        </SelectGroup>
                    </SelectContent>
                </Select>



            </header>

            <section
                ref={parentRef}
                className=' flex-1 w-full overflow-hidden overflow-y-scroll  h-[calc(100vh-3.5rem)]'
            >
                <section
                    style={{
                        height: `${rowVirtualizer.getTotalSize()}px`,
                        width: '100%',
                        position: 'relative',
                    }}
                >

                    {rowVirtualizer.getVirtualItems().map((virtualItem) => (
            <div
              key={virtualItem.key}
              style={{
                position: 'absolute',
                top: 0,
                left: 0,
                width: '100%',
                height: `${virtualItem.size}px`,
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              Row {virtualItem.index}
            </div>
          ))}

                </section>

            </section>


        </main>
    );
};


type EventSearchModeProps = {
    data: z.infer<typeof zLogFrom_Event>
    update: (newData: z.infer<typeof zLogFrom_Event>) => void
}

function EventSearchMode({ data, update }: EventSearchModeProps) {

    return (
        <div className=' flex flex-row gap-2 items-center justify-end'>

            <Select
                items={Object.values(InComingMessageSchemaType.options).map((item) => ({ label: item, value: item }))}
                onValueChange={(v) => {
                    update({ ...data, eventType: v as z.infer<typeof InComingMessageSchemaType> | null })
                }}
            >
                <SelectTrigger className="w-50">
                    <SelectValue placeholder="Event Type" />
                </SelectTrigger>
                <SelectContent>
                    <SelectGroup>
                        {Object.values(InComingMessageSchemaType.options).map((item) => (
                            <SelectItem key={item} value={item}>
                                {item}
                            </SelectItem>
                        ))}
                        <SelectItem key={"null"} value={null}>
                            {"None"}
                        </SelectItem>
                    </SelectGroup>
                </SelectContent>
            </Select>

            <Select
                items={Object.values(moduleTypeIdentifier.options).map((item) => ({ label: item, value: item }))}
                onValueChange={(v) => update({ ...data, dentifier: v as z.infer<typeof moduleTypeIdentifier> | null })}>
                <SelectTrigger className="w-50">
                    <SelectValue placeholder="Module Type" />
                </SelectTrigger>
                <SelectContent>
                    <SelectGroup>
                        {Object.values(moduleTypeIdentifier.options).map((item) => (
                            <SelectItem key={item} value={item}>
                                {item}
                            </SelectItem>
                        ))}

                        <SelectItem key={"null"} value={null}>
                            {"None"}
                        </SelectItem>
                    </SelectGroup>
                </SelectContent>
            </Select>


        </div>
    )
}


type LogSearchModeProps = {
    data: z.infer<typeof zLogFrom_Log>
    update: (newData: z.infer<typeof zLogFrom_Log>) => void
}

function LogSearchMode({ data, update }: LogSearchModeProps) {

    return (
        <div className=' flex flex-row gap-2 items-center justify-end'>
            <Select
                items={Object.values(log_priority.options).map((item) => ({ label: item, value: item }))}
                onValueChange={(v) => {
                    update({ ...data, priority: v as z.infer<typeof log_priority> | null })
                }}
            >
                <SelectTrigger className="w-50">
                    <SelectValue placeholder="Priority" />
                </SelectTrigger>
                <SelectContent>
                    <SelectGroup>
                        {Object.values(log_priority.options).map((item) => (
                            <SelectItem key={item} value={item}>
                                {item}
                            </SelectItem>
                        ))}
                        <SelectItem key={"null"} value={null}>
                            {"None"}
                        </SelectItem>
                    </SelectGroup>
                </SelectContent>
            </Select>


        </div>
    )

}