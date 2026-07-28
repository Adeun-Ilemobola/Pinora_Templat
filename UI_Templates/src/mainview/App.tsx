import { useEffect, useMemo, useState } from "react"

import { Badge } from "@/components/ui/badge"
import { Card, CardContent, CardHeader } from "@/components/ui/card"

import { Separator } from "@/components/ui/separator"

import Layout from "./lib/Layout"
import { Label } from "./components/ui/label"
import { electroview } from "./electrobun";
import { toast } from "sonner"
import { SerialDeviceInfo } from "../shared/rpc"
import { Button } from "./components/ui/button"
import { useModuleStore } from "../Runtime/ModuleStore";
import Liddar from "./Modules/Lidar/view"
import { Input } from "./components/ui/input"
import { MoveUp } from "lucide-react"

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
	const [roundOption , setRoundOption] = useState<"none" | "round" |"floor" | "ceil">("none")
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

	const resultRoundOption = useMemo(()=>{
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
	},[result ,roundOption])

	const processSequence = useMemo(()=>{
		let sequenceMap:number[][]=[]
		let currentStep =0;
		while (currentStep < resultRoundOption.stepValue && currentStep !== resultRoundOption.stepValue  ){
			const sequenceIndex = currentStep % sequence.length;
            const activePattern = sequence[sequenceIndex];
			sequenceMap.push(activePattern)
			currentStep+=1
		}
		return sequenceMap

	},[result , roundOption])

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
			degreesPer: (b* toNumber)
		}))
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

			<Liddar/>

			{/* <div className=" flex flex-row gap-2 p-2 items-center justify-centerring rounded-xl">
				<Button variant={roundOption === "none" ? "default" :"outline"} onClick={()=>setRoundOption("none")}>
					none
				</Button>
				<Button variant={roundOption === "round" ? "default" :"outline"} onClick={()=>setRoundOption("round")}>
					round
				</Button>
				<Button variant={roundOption === "floor" ? "default" :"outline"} onClick={()=>setRoundOption("floor")}>
					floor
				</Button>
				<Button variant={roundOption === "ceil" ? "default" :"outline"} onClick={()=>setRoundOption("ceil")}>
					ceil
				</Button>
			</div>
			<div className=" flex flex-row gap-4">
				<div className=" ring rounded-xl p-3 flex flex-col gap-3.5 justify-center items-center">
					<div className="flex flex-col gap-2 items-center">
						<Label>angle To Steps</Label>
						<div className="flex flex-row gap-2 items-center">
							<Input value={degrees} onChange={(e) => setdegrees(e.target.value)} />
							<Button onClick={() => { angleToSteps() }}>To step</Button>
						</div>
					</div>

					<h1>
						{resultRoundOption.stepValue}
					</h1>


				</div>


				<div className="ring rounded-xl p-3 flex flex-col gap-3.5 justify-center items-center">
					<div className="flex flex-col gap-2 items-center">
						<Label>Steps To angle</Label>
						<div className="flex flex-row gap-2 items-center">
							<Input value={step} onChange={(e) => setStep(e.target.value)} />
							<Button onClick={() => { degreesPerStep() }}>To step</Button>
						</div>
					</div>

					<h1>
						{resultRoundOption.degreesPer}
					</h1>


				</div>
			</div>
			<h1 className=" text-4xl">
				Sequence Point :{processSequence.length}
			</h1>
			<h1 className=" text-xl">
				Sequence Point :{processSequence}
			</h1> */}

			








		</div>

	)

}

export default App
