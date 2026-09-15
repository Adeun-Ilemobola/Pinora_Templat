import { expect, test } from "bun:test";
import { createElement } from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { useModuleFront } from "../src/lib/Modulefront";
import { LidarView, RegisteredLidarView } from "../src/lib/Modules/lidar";

const bindings = [
  ["Lidar", "lidar"],
  ["Servo", "servo_x"],
  ["Servo", "servo_y"],
  ["Rangefinder", "rangefinder"],
] as const;
const props = {
  id: "lidar",
  servoX_id: "servo_x",
  servoY_id: "servo_y",
  range_id: "rangefinder",
};
const registeredProps = {
  moduleId: "lidar",
  servoX_id: "servo_x",
  servoY_id: "servo_y",
  range_id: "rangefinder",
};

test("LiDAR renders only with all four explicit lookups and correctly typed stores", () => {
  const saved = useModuleFront.getState();
  const snapshot = useModuleFront.getInitialState();
  const savedSnapshot = { ...snapshot };
  const render = (registered = false) => {
    Object.assign(snapshot, useModuleFront.getState());
    return renderToStaticMarkup(
      registered
        ? createElement(RegisteredLidarView, registeredProps)
        : createElement(LidarView, props),
    );
  };
  try {
    useModuleFront.setState({
      ModuleRegistry: {},
      LookUpId: {},
      logs: [],
      PortStat: "Connected",
    });
    for (const [kind, id] of bindings) {
      expect(render()).toBe("");
      useModuleFront
        .getState()
        .RegistrationEvent({
          id,
          module_type: kind,
          lool_up_id: id,
          parent_id: kind === "Lidar" ? "" : "lidar",
        });
    }
    const complete = useModuleFront.getState();
    expect(render()).toContain("Scan map");
    expect(render()).toContain("X pivot");
    expect(render()).toContain("Y pivot");
    expect(render()).toContain("Not reported");
    for (const [, id] of bindings) {
      const lookups = { ...complete.LookUpId };
      delete lookups[id];
      useModuleFront.setState({ LookUpId: lookups });
      expect(render()).toBe("");
      useModuleFront.setState({ LookUpId: complete.LookUpId });

      const registry = { ...complete.ModuleRegistry };
      delete registry[id];
      useModuleFront.setState({ ModuleRegistry: registry });
      expect(render()).toBe("");
      expect(render(true)).toBe("");

      registry[id] =
        complete.ModuleRegistry[id === "lidar" ? "servo_x" : "lidar"];
      useModuleFront.setState({ ModuleRegistry: registry });
      expect(render()).toBe("");
      expect(render(true)).toBe("");
      useModuleFront.setState({ ModuleRegistry: complete.ModuleRegistry });
    }
    expect(render(true)).toContain("Scan map");
  } finally {
    useModuleFront.setState(saved, true);
    Object.assign(snapshot, savedSnapshot);
  }
});
