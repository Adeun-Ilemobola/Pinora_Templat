import { Input } from '@/components/ui/input'
import { Slider } from '@/components/ui/slider'
import { Command } from '@/lib/ModuleCommand';
import { ServoModule } from '@/lib/ModuleDefinitionSchema';
import { memo, useEffect, useState } from 'react';
import z from 'zod';
import ModuleCore from './ModuleCore';


function clamp(value: number, min: number, max: number) {
  return Math.round(Math.min(Math.max(value, min), max));
}


function NumberField({
  label,
  value,
  onChange,
  onCommit,
}: {
  label: string;
  value: number;
  onChange: (value: number) => void;
  onCommit: () => void;
}) {
  const [draft, setDraft] = useState(String(value));

  useEffect(() => {
    setDraft(String(value));
  }, [value]);

  function handleChange(raw: string) {
    // Allow empty typing and negative typing states:
    // "", "-", "-3", "35"
    if (!/^-?\d*$/.test(raw)) return;

    setDraft(raw);

    // Do not commit incomplete values yet
    if (raw === "" || raw === "-") return;

    onChange(Number(raw));
  }

  function handleCommit() {
    // If user leaves it empty or just "-", restore real value
    if (draft === "" || draft === "-") {
      setDraft(String(value));
      return;
    }

    onChange(Number(draft));
    onCommit();
  }

  return (
    <div className="flex flex-col gap-1">
      <label className="text-xs text-accent-foreground/35">
        {label}
      </label>

      <Input
        type="text"
        inputMode="numeric"
        value={draft}
        onChange={(e) => handleChange(e.target.value)}
        onKeyDown={(e) => {
          if (e.key !== "Enter") return;
          e.preventDefault();
          handleCommit();
        }}
        onBlur={handleCommit}
      />
    </div>
  );
}


type ServoCardProps = {
  module: z.infer<typeof ServoModule>;
  sendCommand: (command: Command) => Promise<void>;
  Disable:boolean
};

export const ServoCard = memo(function ServoCard({ module, sendCommand  , Disable= false}: ServoCardProps) {
  const [pivot, setPivot] = useState(module.state.angle);
  const [minPivot, setMinPivot] = useState(-90);
  const [maxPivot, setMaxPivot] = useState(90);

  useEffect(() => {
    setPivot(module.state.angle);
  }, [module.state.angle]);

  function sendAngle(angle = pivot) {
    if (Disable){return}
    if (module.parent_id.length > 5) {
      void sendCommand({
        id: module.parent_id,
        module_type: "Lidar",
        payload: {
          command: "ChangeMotorAngle",
          id: module.id,
          step: clamp(angle, minPivot, maxPivot)
        }
      })
      return
    }
    void sendCommand({
      id: module.id,
      module_type: "Servo",
      payload: {
        command: "SetAngle",
        angle: clamp(angle, minPivot, maxPivot),
      }
    })
  }

  // function sendMinPivot() {
  //   void sendCommand({
  //     id: module.id,
  //     module_type: "Servo",
  //     payload: {
  //       command:""
  //     }
  //   })
  // }

  // function sendMaxPivot() {
  //   void sendCommand({
  //     id: module.id,
  //     module_type: "Servo",
  //     payload: {
  //       SetMaxPivot: {
  //         max_pivot: Math.round(Math.max(maxPivot, minPivot)),
  //       },
  //     }
  //   })
  // }

  return (
    <ModuleCore
      id={module.id}
      manuel_id={module.lool_up_id}
      moduletype={module.module_type}
    >
      <div className=' flex flex-col items-center gap-2.5'>
        <h1 className=' text-3xl text-center'>
          {module.state.angle}&deg;
        </h1>

        <div className='flex w-full flex-col gap-2'>
          <label className='text-xs text-accent-foreground/35'>
            Pivot: {pivot}&deg;
          </label>
          <Slider
          disabled={Disable}
            value={[pivot]}
            min={minPivot}
            max={maxPivot}
            step={1}
            onValueChange={([angle]) => setPivot(angle)}
            onValueCommit={([angle]) => sendAngle(angle)}
            aria-label='Pivot angle'
          />
        </div>

        <div className=' flex flex-row gap-2.5'>
          <NumberField
            label='Min Pivot'
            value={minPivot}
            onChange={setMinPivot}
            onCommit={() => { }}
          />
          <NumberField
            label='Max Pivot'
            value={maxPivot}
            onChange={setMaxPivot}
            onCommit={() => { }}
          />
        </div>
      </div>
    </ModuleCore>
  )
});
