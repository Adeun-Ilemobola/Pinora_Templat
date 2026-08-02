import { Separator } from '@/components/ui/separator';
import { useVirtualizer } from '@tanstack/react-virtual';

import {
    InComingMessageSchemaType,
    moduleTypeIdentifier,
    viewLogSchema,
} from '@shared/Protocol/ModuleDefinitionSchema';

import { useMemo, useRef, useState } from 'react';
import z from 'zod';

import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { useModuleStore } from '@runtime/ModuleStore';

type LogsProps = {};

type LogFromType = 'Event' | 'log' | null;

const items: { label: string; value: LogFromType }[] = [
    { label: 'Event', value: 'Event' },
    { label: 'Log', value: 'log' },
    { label: 'None', value: null },
];

const zLogFrom_Event = z.object({
    Type: z.literal('Event'),
    eventType: InComingMessageSchemaType.nullable(),
    dentifier: moduleTypeIdentifier.nullable(),
});

const log_priority = z.enum(['Low', 'Medium', 'High', 'Critical']);

const zLogFrom_Log = z.object({
    Type: z.literal('log'),
    priority: log_priority.nullable(),
});

const zLogFrom = z.discriminatedUnion('Type', [
    zLogFrom_Event,
    zLogFrom_Log,
]);

type zLogFromType = z.infer<typeof zLogFrom>;
type logShape = z.infer<typeof viewLogSchema>

type ModuleEventData = Extract<
    logShape["data"],
    { type: "ModuleEvent" }
>;

type RegistrationData = Extract<
    logShape["data"],
    { type: "Registration" }
>;

type SystemData = Extract<
    logShape["data"],
    { type: "System" }
>;

export const LogsBox = ({ }: LogsProps) => {
    const logs = useModuleStore(state => state.logs)
    const parentRef = useRef(null);

    const [searchMode, setSearchMode] = useState<zLogFromType | null>(null);

    const logList = useMemo(() => {

        if (!searchMode) {

            return logs;
        }

        switch (searchMode.Type) {
            case 'Event': {
                const baseEventFilter = logs.filter(item =>
                    item.data.type !== "ModuleEvent" ||
                    item.data.payload.module_type !== "SysLog"
                )
                if (!searchMode.dentifier && !searchMode.eventType) {
                    return baseEventFilter
                }


                const coreEventFilter = baseEventFilter.filter(item => {
                    if (searchMode.eventType && searchMode.eventType !== item.data.type) {
                        return false
                    }

                    if (searchMode.dentifier) {
                        return (
                            item.data.type === "ModuleEvent" ||
                            item.data.type === "Registration"
                        ) && searchMode.dentifier === item.data.payload.module_type
                    }

                    return true
                })

                return coreEventFilter
            }

            case 'log': {
                const baseLogFilter = logs.filter(item => (

                    item.data.type === "ModuleEvent" &&
                    item.data.payload.module_type === "SysLog"
                ))
                if (!searchMode.priority) {
                    return baseLogFilter
                }

                const coreLogFilter = baseLogFilter.filter(item => {
                    if (searchMode.priority && item.data.type === "ModuleEvent" && item.data.payload.module_type === "SysLog") {
                        return searchMode.priority === item.data.payload.event.priority

                    }
                    return false

                })


                return coreLogFilter



            }
        }

    }, [logs, searchMode]);

    const rowVirtualizer = useVirtualizer({
        count: logList.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => 35,
    });

    function onSearchModeChange(mode: LogFromType) {
        switch (mode) {
            case 'Event': {
                setSearchMode({
                    Type: 'Event',
                    eventType: null,
                    dentifier: null,
                });

                break;
            }

            case 'log': {
                setSearchMode({
                    Type: 'log',
                    priority: null,
                });

                break;
            }

            default: {
                setSearchMode(null);
                break;
            }
        }
    }

    const LoadMode = useMemo(() => {
        if (!searchMode) {
            return <></>;
        }

        switch (searchMode.Type) {
            case 'Event': {
                return (
                    <EventSearchMode
                        data={searchMode}
                        update={(value) => {
                            setSearchMode((previous) => {
                                if (!previous) {
                                    return previous;
                                }

                                return {
                                    ...previous,
                                    ...value,
                                };
                            });
                        }}
                    />
                );
            }

            case 'log': {
                return (
                    <LogSearchMode
                        data={searchMode}
                        update={(value) => {
                            setSearchMode((previous) => {
                                if (!previous) {
                                    return previous;
                                }

                                return {
                                    ...previous,
                                    ...value,
                                };
                            });
                        }}
                    />
                );
            }
        }
    }, [searchMode]);
    function renderLogData(data: logShape["data"]) {
        switch (data.type) {
            case "ModuleEvent":
                return <EventCard data={data} />;

            case "Registration":
                return <RegistrationCard data={data} />;

            case "System":
                return <SystemInfoCard data={data} />;

            default:
                return null;
        }
    }

    return (
        <main className="flex min-h-0 w-full flex-1 flex-col bg-background">
            <header className="sticky top-13 z-19 flex min-h-14 shrink-0 items-center border-b bg-background/95 px-4 py-2 backdrop-blur">
                <div className="flex w-full items-center justify-between gap-4">
                    <div className="min-w-0">
                        <h2 className="text-sm font-semibold">Logs</h2>
                        <p className="text-xs text-muted-foreground">
                            View and filter incoming activity
                        </p>
                    </div>

                    <div className="flex min-w-0 items-center justify-end gap-2">
                        {LoadMode}

                        {searchMode && (
                            <Separator
                                orientation="vertical"
                                className="mx-1 h-6"
                            />
                        )}

                        <Select

                            items={items}
                            onValueChange={(value) => {
                                onSearchModeChange(value as LogFromType);
                            }}
                        >
                            <SelectTrigger className="w-40">
                                <SelectValue placeholder="Filter mode" />
                            </SelectTrigger>

                            <SelectContent side="bottom"
                                sideOffset={6}
                                align="end"
                                alignItemWithTrigger={false}>
                                <SelectGroup>
                                    {items.map((item) => (
                                        <SelectItem
                                            key={item.value}
                                            value={item.value}
                                        >
                                            {item.label}
                                        </SelectItem>
                                    ))}
                                </SelectGroup>
                            </SelectContent>
                        </Select>
                    </div>
                </div>
            </header>

            <section
                ref={parentRef}
                className="h-[calc(100vh-3.5rem)] min-h-0 w-full flex-1 overflow-y-auto"
            >
                <section
                    className="relative w-full"
                    style={{
                        height: `${rowVirtualizer.getTotalSize()}px`,
                    }}
                >
                    {rowVirtualizer
                        .getVirtualItems()
                        .map((virtualItem) => {
                            const coreData = logList[virtualItem.index]



                            return (
                                <div
                                    key={virtualItem.key}
                                    className="absolute left-0 top-0 flex w-full items-center border-b px-4 text-sm transition-colors hover:bg-muted/50"
                                    style={{
                                        height: `${virtualItem.size}px`,
                                        transform: `translateY(${virtualItem.start}px)`,
                                    }}
                                >
                                    {renderLogData(coreData.data)}

                                </div>
                            )
                        })}
                </section>
            </section>
        </main>
    );
};

type EventSearchModeProps = {
    data: z.infer<typeof zLogFrom_Event>;
    update: (newData: z.infer<typeof zLogFrom_Event>) => void;
};

function EventSearchMode({
    data,
    update,
}: EventSearchModeProps) {
    return (
        <div className="flex min-w-0 items-center justify-end gap-2">
            <Select
                items={Object.values(
                    InComingMessageSchemaType.options,
                ).map((item) => ({
                    label: item,
                    value: item,
                }))}
                onValueChange={(value) => {
                    update({
                        ...data,
                        eventType:
                            value as z.infer<
                                typeof InComingMessageSchemaType
                            > | null,
                    });
                }}
            >
                <SelectTrigger className="w-44">
                    <SelectValue placeholder="Event type" />
                </SelectTrigger>

                <SelectContent
                    side="bottom"
                    sideOffset={6}
                    align="end"
                    alignItemWithTrigger={false}
                >
                    <SelectGroup>
                        {Object.values(
                            InComingMessageSchemaType.options,
                        ).map((item) => (
                            <SelectItem key={item} value={item}>
                                {item}
                            </SelectItem>
                        ))}

                        <SelectItem key="null" value={null}>
                            None
                        </SelectItem>
                    </SelectGroup>
                </SelectContent>
            </Select>

            <Select
                items={Object.values(
                    moduleTypeIdentifier.options,
                ).map((item) => ({
                    label: item,
                    value: item,
                }))}
                onValueChange={(value) => {
                    update({
                        ...data,
                        dentifier:
                            value as z.infer<
                                typeof moduleTypeIdentifier
                            > | null,
                    });
                }}
            >
                <SelectTrigger className="w-44">
                    <SelectValue placeholder="Module type" />
                </SelectTrigger>

                <SelectContent
                    side="bottom"
                    sideOffset={6}
                    align="end"
                    alignItemWithTrigger={false}
                >
                    <SelectGroup>
                        {Object.values(
                            moduleTypeIdentifier.options,
                        ).map((item) => (
                            <SelectItem key={item} value={item}>
                                {item}
                            </SelectItem>
                        ))}

                        <SelectItem key="null" value={null}>
                            None
                        </SelectItem>
                    </SelectGroup>
                </SelectContent>
            </Select>
        </div>
    );
}

type LogSearchModeProps = {
    data: z.infer<typeof zLogFrom_Log>;
    update: (newData: z.infer<typeof zLogFrom_Log>) => void;
};

function LogSearchMode({
    data,
    update,
}: LogSearchModeProps) {
    return (
        <div className="flex min-w-0 items-center justify-end gap-2">
            <Select
                items={Object.values(log_priority.options).map(
                    (item) => ({
                        label: item,
                        value: item,
                    }),
                )}
                onValueChange={(value) => {
                    update({
                        ...data,
                        priority:
                            value as z.infer<
                                typeof log_priority
                            > | null,
                    });
                }}
            >
                <SelectTrigger className="w-44">
                    <SelectValue placeholder="Priority" />
                </SelectTrigger>

                <SelectContent
                    side="bottom"
                    sideOffset={6}
                    align="end"
                    alignItemWithTrigger={false}
                >
                    <SelectGroup>
                        {Object.values(log_priority.options).map(
                            (item) => (
                                <SelectItem
                                    key={item}
                                    value={item}
                                >
                                    {item}
                                </SelectItem>
                            ),
                        )}

                        <SelectItem key="null" value={null}>
                            None
                        </SelectItem>
                    </SelectGroup>
                </SelectContent>
            </Select>
        </div>
    );
}


function EventCard({ data }: { data: ModuleEventData }) {

    if (data.payload.module_type === "SysLog") {
        return (
            <div>
                System Log for  debugging == {data.payload.event.text}
            </div>
        )
    }

    return (
        <div>
            The state event change === {data.payload.event.event_type}
        </div>
    )

}



function RegistrationCard({ data }: { data: RegistrationData }) {

    return (
        <div>
            For registered modules event === {data.payload.module_type}
        </div>
    )

}

function SystemInfoCard({ data }: { data: SystemData }) {

    return (
        <div>
            For SystemInfo  modules event === {data.payload.current_free_heap}
        </div>
    )

}
