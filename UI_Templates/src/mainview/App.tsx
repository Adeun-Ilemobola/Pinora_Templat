import { useEffect, useState } from "react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader } from "@/components/ui/card"

import { Separator } from "@/components/ui/separator"

import { Label } from "@/components/ui/label"
import { electroview } from "@/electrobun";
import { toast } from "sonner"
import { SerialDeviceInfo } from "@shared/rpc"
import { Button } from "@/components/ui/button"
import { useModuleStore } from "@runtime/ModuleStore";
import { selectModule } from "@shared/Protocol/ModuleDefinitionSchema"
import { StepperCard } from "./Modules/stepper/view"
import { ImuCard } from "./Modules/IMU/view"

function App() {
	const [ports, setPorts] = useState<SerialDeviceInfo[]>([])
	const portinfo = useModuleStore((state) => state.portInfo)
	 const Stepper = useModuleStore((state) => selectModule(state, "stepperX", "StepperMotor"))
	  const Imu = useModuleStore((state) => selectModule(state, "MPu", "Imu"))
	  const CoD = useModuleStore((state) => state.sendCommand)

	  useEffect(() => {
		const load = async () => {
			const listPorts = await electroview.rpc?.request.getAvailablePorts()
			if (listPorts) {
				setPorts(listPorts)
			}
		}
		load().catch(err => {
			console.error(err)
		})
	}, [])

	

	

	async function StartConnection(portId: string) {
		try {
			console.log(`ui port :${portId}`)
			await electroview.rpc?.request.openPort({ port: portId })

		} catch (error) {
			console.error("failed to open ESP port :", error)
			toast.error("failed to open ESP port")

		}
	}




	return (
		<div className=" flex flex-col gap-2 flex-1 h-full w-full">

			<div className=" grid grid-cols-2 gap-3.5 p-3.5 ">

				<Card className=" shrink-0">
					<CardHeader>
						<div className="flex flex-row flex-wrap items-center gap-3">
							<h1 className=" text-3xl">Available port</h1>
							<Badge variant={portinfo.status === "connected" ? "default" : "destructive"}> ESP {portinfo.status === "connected" ? "connected" : "not connected"}</Badge>
							<Separator orientation="vertical" />
							<div className=" flex-1 flex flex-col gap-0.5 p-1">
								<Label>Time</Label>
								<span>00:00:00</span>
							</div>
						</div>
						<Separator />
					</CardHeader>
					<CardContent>
						<div className=" flex flex-col gap-2.5  h-72 overflow-hidden overflow-y-auto ">
							{ports.map((p, i) => {
								return (
									<Button variant={"outline"} className={" w-full"} key={i} onClick={() => { StartConnection(p.path) }}>
										{p.path}
									</Button>
								)
							})}

						</div>
					</CardContent>
				</Card>

				<Card className="shrink-0">
					<CardHeader>
						<div className="flex flex-row flex-wrap items-center gap-3">
							<h1 className=" text-3xl">ESP-32 info</h1>
							<Badge variant={portinfo.status === "connected" ? "default" : "destructive"}> ESP {portinfo.status === "connected" ? "connected" : "not connected"}</Badge>
						</div>
						<Separator />
					</CardHeader>
					<CardContent>
						<div className="shrink-0 flex flex-col gap-2.5 justify-center h-72   overflow-hidden overflow-y-auto">

						</div>
					</CardContent>
				</Card>

			</div>

			{/* <Liddar/> */}

			{
				Stepper && <StepperCard
					module={Stepper}
					sendCommand={CoD}
					Disable={portinfo.status === "connected" ? false : true}
				/>
			}

			{Imu && (
				<div className="p-3.5">
					<ImuCard module={Imu} />
				</div>
			)}


		</div>

	)

}

export default App
