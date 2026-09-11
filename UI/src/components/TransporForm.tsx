import { useEffect, useState } from "react";
import { Card, CardHeader, CardContent, CardTitle } from "./ui/card";
import { Button } from "./ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Field, FieldDescription, FieldError, FieldLabel } from "./ui/field";
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select"
import { invoke } from "@tauri-apps/api/core";
import { useModuleFront } from "@/lib/Modulefront";
const items = [
    { label: "115200", value: "115200" },
    { label: "230400", value: "230400" },
    { label: "460800", value: "460800" },
    { label: "921600", value: "921600" },
]

export function TransporForm() {
    const Start = useModuleFront(state => state.Connect);
    const Disconnect = useModuleFront(state => state.Disconnect);
    const PortStat = useModuleFront(state => state.PortStat);
    const [name, setName] = useState("");
    const [rate, setRate] = useState("");
    const [portList, setPortList] = useState<string[]>([]);


    useEffect(() => {
        RefreshPortList();
    }, []);

    function StartConnection() {
        console.log("Starting connection with", { name, rate });
        if (!name || !rate) {
            console.error("Name and rate are required.");
            return;
        }
        if (isNaN(parseInt(rate))) {
            console.error("Rate must be a number.");
            return;
        }
        Start(name, parseInt(rate));
    }
    function StopConnection() {
        Disconnect();
    }

    function RefreshPortList() {
        console.log("Refreshing port list");
        invoke("get_available_ports")
            .then((result) => {
                console.log("Refresh result:", result);
                setPortList(result as string[]);
            })
            .catch((error) => {
                console.error("Refresh error:", error);
            });
    }
  


    return (
        <Card>
            <CardHeader className=" flex flex-row gap-2.5">
                <CardTitle>Transport Form</CardTitle>
                <Button onClick={RefreshPortList}>Refresh Ports</Button>
            </CardHeader>
            <CardContent>
                <Tabs defaultValue="serial" className="w-70 h-55">
                    <TabsList>
                        <TabsTrigger value="serial">Serial</TabsTrigger>
                        <TabsTrigger value="network">Network</TabsTrigger>
                    </TabsList>
                    <TabsContent value="serial">
                        <div className=" flex flex-col gap-1.5 justify-center items-center">
                            <Field>
                                <FieldLabel htmlFor="input-name">Name</FieldLabel>
                                <Select disabled={PortStat === "Connected"} items={items} id="input-name" value={name} onValueChange={(value) => setName(value || "")}>
                                    <SelectTrigger className="">
                                        <SelectValue placeholder="Name" />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectGroup>
                                            {portList.map((item) => (
                                                <SelectItem key={item} value={item}>
                                                    {item}
                                                </SelectItem>
                                            ))}
                                        </SelectGroup>
                                    </SelectContent>
                                </Select>

                            </Field>

                            <Field>
                                <FieldLabel htmlFor="input-rate">Rate</FieldLabel>
                                <Select disabled={PortStat === "Connected"} items={items} id="input-rate" value={rate} onValueChange={(value) => setRate(value || "")}>
                                    <SelectTrigger className="">
                                        <SelectValue placeholder="Rate" />
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

                            </Field>

                            <Button variant={PortStat === "Connected" ? "destructive" : "default"} onClick={PortStat === "Connected" ? StopConnection : StartConnection}> 
                                {PortStat === "Connected" ? "Disconnect" : "Connect"}
                                </Button>
                        </div>


                    </TabsContent>
                    <TabsContent value="network">
                        <p>Coming soon...</p>
                    </TabsContent>
                </Tabs>
            </CardContent>
        </Card>
    );
}