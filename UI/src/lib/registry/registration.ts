import { Registration } from "../protocol/registration";

/** True when the registration names a parent module (children are controlled by it). */
export function registrationHasParent(registration: Registration): boolean {
  // lidar.rs creates two servos and a rangefinder; joystick.rs owns a button
  // (the joystick implementation is currently excluded from the firmware build).
  // if (
  //   registration.module_type === "Lidar" ||
  //   registration.module_type === "JoyStick"
  // ) {
  //   return false;
  // }
  // ModuleCore::new serializes None as an empty string. Also tolerate older
  // registrations with missing/null parent_id without changing the wire schema.
  return (
    typeof registration.parent_id === "string" &&
    registration.parent_id.length > 0
  );
}
