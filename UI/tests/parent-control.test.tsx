import { beforeEach, expect, test } from "bun:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { registrationHasParent, useModuleFront } from "../src/lib/Modulefront";
import { RegisteredLedView } from "../src/lib/Modules/Led";
import { RegisteredServoView } from "../src/lib/Modules/servo";
import { RegisteredRangefinderView } from "../src/lib/Modules/rangefinder";
import { RegisteredRfidView } from "../src/lib/Modules/rfid";
import { RegisteredStepperView } from "../src/lib/Modules/stepper";
import type { Registration } from "../src/lib/protocol/registration";

const leaves = [
  "Led",
  "Servo",
  "Button",
  "Imu",
  "Rangefinder",
  "RemoteReceiver",
  "Rfid",
  "StepperMotor",
] as const;
const registration = (
  module_type: Registration["module_type"],
  parent_id: unknown,
  id: string = module_type,
) => ({ id, module_type, lool_up_id: id, parent_id }) as Registration;

beforeEach(() => {
  useModuleFront.setState({
    ModuleRegistry: {},
    LookUpId: {},
    PortStat: "Connected",
  });
});

for (const kind of leaves) {
  test(`${kind}: current registration determines ownership, including firmware's empty sentinel`, () => {
    for (const parent of [
      undefined,
      null,
      "",
      "parent-a",
      "",
      "parent-b",
      null,
    ]) {
      useModuleFront.getState().RegistrationEvent(registration(kind, parent));
      expect(
        useModuleFront.getState().ModuleRegistry[kind].getState().hasParent,
      ).toBe(typeof parent === "string" && parent.length > 0);
    }
  });
}

test("verified composite types are exempt; LiDAR and each child resolve independently", () => {
  for (const kind of ["Lidar", "JoyStick"] as const) {
    expect(registrationHasParent(registration(kind, "another-parent"))).toBe(
      false,
    );
  }
  for (const [kind, id, parent] of [
    ["Lidar", "lidar", "outer"],
    ["Servo", "x", "lidar"],
    ["Servo", "y", "lidar"],
    ["Rangefinder", "range", "lidar"],
    ["Servo", "free", ""],
  ] as const) {
    useModuleFront.getState().RegistrationEvent(registration(kind, parent, id));
  }
  const registry = useModuleFront.getState().ModuleRegistry;
  expect(registry.lidar.getState().hasParent).toBe(false);
  for (const id of ["x", "y", "range"])
    expect(registry[id].getState().hasParent).toBe(true);
  expect(registry.free.getState().hasParent).toBe(false);
  useModuleFront.setState({ ModuleRegistry: {}, LookUpId: {} });
  useModuleFront.getState().RegistrationEvent(registration("Servo", "", "x"));
  expect(useModuleFront.getState().ModuleRegistry.x.getState().hasParent).toBe(
    false,
  );
});

for (const [kind, View, telemetry] of [
  ["Led", RegisteredLedView, "Reported brightness"],
  ["Servo", RegisteredServoView, "Reported angle"],
  ["Rangefinder", RegisteredRangefinderView, "Last valid distance"],
  ["Rfid", RegisteredRfidView, "Last scanned tag"],
  ["StepperMotor", RegisteredStepperView, "Reported total angle"],
] as const) {
  test(`${kind}: rendered manual controls are disabled while telemetry remains visible`, () => {
    const initial = useModuleFront.getInitialState();
    const saved = { ...initial };
    // SSR snapshot of the registered fixture, without a browser or hardware transport.

    try {
      for (const parent of ["parent", ""]) {
        useModuleFront.getState().RegistrationEvent(registration(kind, parent));
        Object.assign(initial, useModuleFront.getState());
        const html = renderToStaticMarkup(
          createElement(View, { moduleId: kind }),
        );
        expect(html).toContain(telemetry);
        expect(html.includes("Parent controlled")).toBe(Boolean(parent));
        const controls = (
          html.match(/<(?:button|input)\b[^>]*>/g) ?? []
        ).filter(
          (tag) =>
            !tag.includes('style="display:none"') &&
            !tag.includes('type="hidden"'),
        );
        const sliders =
          html.match(/<[^>]+role="(?:slider|spinbutton)"[^>]*>/g) ?? [];
        expect(controls.length + sliders.length).toBeGreaterThan(0);
        if (parent) {
          for (const tag of controls)
            expect(tag).toMatch(/\sdisabled(?:=""|\s|>)/);
          for (const tag of sliders)
            expect(tag).toMatch(/aria-disabled="true"|data-disabled=""/);
        } else {
          expect(
            [...controls, ...sliders].some(
              (tag) =>
                !/\sdisabled(?:=""|\s|>)|aria-disabled="true"|data-disabled=""/.test(
                  tag,
                ),
            ),
          ).toBe(true);
        }
      }
    } finally {
      Object.assign(initial, saved);
    }
  });
}
