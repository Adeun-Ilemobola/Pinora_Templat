import { useEffect, useMemo, useState } from "react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader } from "@/components/ui/card"

import { Separator } from "@/components/ui/separator"

import Layout from "@/lib/Layout"
import { Label } from "@/components/ui/label"
import { electroview } from "@/electrobun";
import { toast } from "sonner"
import { SerialDeviceInfo } from "@shared/rpc"
import { Button } from "@/components/ui/button"
import { useModuleStore } from "@runtime/ModuleStore";
import Liddar from "@modules/Lidar/view"
import { Input } from "@/components/ui/input"
import { MoveUp } from "lucide-react"
import z from "zod"
import { PointSchema } from "@shared/Protocol/ModuleEven"

const sequence = [
	[1, 0, 0, 0],
	[1, 1, 0, 0],
	[0, 1, 0, 0],
	[0, 1, 1, 0],
	[0, 0, 1, 0],
	[0, 0, 1, 1],
	[0, 0, 0, 1],
	[1, 0, 0, 1],
];

type HoverPoint = {
	x: number,
	y: number,
	zoneX: number,
	zoneY: number,
}

const stepsPerRevolution = 4096;
function App() {
	const [ports, setPorts] = useState<SerialDeviceInfo[]>([])
	const [openedPort, setOpenedPort] = useState(false)
	const portinfo = useModuleStore((state) => state.portInfo)
	const [step, setStep] = useState("0")
	const [degrees, setdegrees] = useState("0")
	const [result, setResult] = useState({
		stepValue: 0.0,
		degreesPer: 0.0
	})
	const [hoverPoint, UpdateHoverPoint] = useState<HoverPoint | null>(null)


	const [roundOption, setRoundOption] = useState<"none" | "round" | "floor" | "ceil">("none")
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

	// const [sequenceMap , setSequenceMap] = useState<number[][]>([])

	async function StartConnection(portId: string) {
		try {
			console.log(`ui port :${portId}`)
			await electroview.rpc?.request.openPort({ port: portId })

		} catch (error) {
			console.error("failed to open ESP port :", error)
			toast.error("failed to open ESP port")

		}
	}

	const resultRoundOption = useMemo(() => {
		switch (roundOption) {
			case "ceil":
				return {
					stepValue: Math.ceil(result.stepValue),
					degreesPer: Math.ceil(result.degreesPer)
				}
			case "floor":
				return {
					stepValue: Math.floor(result.stepValue),
					degreesPer: Math.floor(result.degreesPer)
				}

			case "round":
				return {
					stepValue: Math.round(result.stepValue),
					degreesPer: Math.round(result.degreesPer)
				}

			case "none":
				return {
					stepValue: result.stepValue,
					degreesPer: result.degreesPer
				}


			default:
				return {
					stepValue: result.stepValue,
					degreesPer: result.degreesPer
				}
		}
	}, [result, roundOption])

	const processSequence = useMemo(() => {
		let sequenceMap: number[][] = []
		let currentStep = 0;
		while (currentStep < resultRoundOption.stepValue && currentStep !== resultRoundOption.stepValue) {
			const sequenceIndex = currentStep % sequence.length;
			const activePattern = sequence[sequenceIndex];
			sequenceMap.push(activePattern)
			currentStep += 1
		}
		return sequenceMap

	}, [result, roundOption])

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

	function PointerLeave(ev: React.PointerEvent<HTMLDivElement>) {
		UpdateHoverPoint(null)

	}
	const root_angle = useMemo(() => {
		if (!hoverPoint) {
			return 0.0
		}
		const radians = Math.atan2((hoverPoint?.y - hoverPoint.zoneY / 2), (hoverPoint.x - hoverPoint.zoneX / 2));
		const degrees = radians * (360 / Math.PI);

		return degrees
	}, [hoverPoint])




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

			<div
				className="h-70 w-70 relative ring ring-fuchsia-600 rounded-full"
				onPointerMove={PointerMove}
				onPointerLeave={PointerLeave}
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








		</div>

	)

}

export default App
