



import {ButtonModule} from '@/components/Modules/ButtonModule'
import { LedCard } from '@/components/Modules/led'
import { useModuleStore } from '@/lib/ModuleStore'
import { ServoCard } from '@/components/Modules/Servo'
import { RangefinderCard } from '@/components/Modules/Rangefinder'


export default function Devices() {
  const modules = useModuleStore((state) => state.modules)
  const sendCommand = useModuleStore((state) => state.sendCommand)

  return (
    <div className=' flex flex-col h-[88dvh] gap-3 p-3 overflow-y-auto'>
      {Object.values(modules).map(item => {
        switch (item.module_type) {
          case "Led":
            return <LedCard key={item.id} module={item} sendCommand={sendCommand} />

          case "Button":
            return <ButtonModule key={item.id} module={item} />

          case "Servo":
            return <ServoCard key={item.id} module={item} sendCommand={sendCommand} Disable={true} />

          case "Rangefinder":
            return <RangefinderCard key={item.id} module={item} sendCommand={sendCommand} />


          default:
            return null;
        }
      })}



    </div>
  )
}
