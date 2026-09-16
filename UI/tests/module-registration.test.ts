import { beforeEach, expect, test } from "bun:test";
import { useModuleFront } from "../src/lib/Modulefront";
import type { EventPackage } from "../src/lib/protocol/event";

beforeEach(() => {
  useModuleFront.setState({ ModuleRegistry: {}, LookUpId: {} });
});

const cases = [
  ["Led", { Led: { Brightness: { level: 42 } } }, 42],
  ["Button", { Button: { Ckick: {} } }, { clickCount: 1 }],
  ["Imu", { Imu: { event_type: "Mode", mode: "Collecting" } }, { Mode: "Collecting" }],
  ["RemoteReceiver", { RemoteReceiver: { Click: { key: "Power" } } }, { key: "Power", clickCount: 1 }],
  ["Rangefinder", { Rangefinder: { Range: { millimeters: 125 } } }, { Range: { millimeters: 125 } }],
  ["Rfid", { Rfid: { GetMode: { mode: "Read" } } }, { GetMode: { mode: "Read" } }],
  ["StepperMotor", { StepperMotor: { GetOrigin: { origin: 12 } } }, { GetOrigin: { origin: 12 } }],
  ["Lidar", { Lidar: { Target: { point: { x: 12, y: 13 } } } }, { Target: { point: { x: 12, y: 13 } } }],
  ["Servo", { Servo: { GetAngle: { angle: 30 } } }, { Angle: 30 }],
] as const;

for (const [kind, event, expected] of cases) {
  test(`${kind}: registration supplies identity, telemetry updates only its own instance`, () => {
    const front = useModuleFront.getState();
    for (const id of ["first", "second"]) {
      front.RegistrationEvent({ id, module_type: kind, lool_up_id: `lookup-${id}`, parent_id: "" });
    }
    const { ModuleRegistry, LookUpId } = useModuleFront.getState();
    const first = ModuleRegistry.first;
    const second = ModuleRegistry.second;
    expect(first.getState()).toMatchObject({ id: "first", kind, look_up_id: "lookup-first", hasParent: false });
    expect(LookUpId["lookup-first"]).toBe("first");
    const untouched = structuredClone(second.getState().state);
    front.ModuleEvent({ id: "first", event } as EventPackage);
    if (typeof expected === "number") expect(first.getState().state).toBe(expected);
    else expect(first.getState().state).toMatchObject(expected);
    expect(second.getState().state).toEqual(untouched);
  });
}
