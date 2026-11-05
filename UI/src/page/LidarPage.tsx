import { LidarView } from "@/lib/Modules/lidar";
import { useModuleFront } from "@/lib/Modulefront";

export default function LidarPage() {
  // Found by kind, so the page does not depend on what the firmware named the LiDAR.
  const lookUpId = useModuleFront((state) => {
    const entry = Object.entries(state.LookUpId).find(
      ([, id]) => state.ModuleRegistry[id]?.getState().kind === "Lidar",
    );
    return entry?.[0] ?? null;
  });
  if (!lookUpId) {
    return (
      <p className="text-sm text-muted-foreground">
        No LiDAR has registered. Enable it in the firmware and connect the device.
      </p>
    );
  }
  return (
    <LidarView
      id={lookUpId}
      servoX_id="servo_x"
      servoY_id="servo_y"
      range_id="rangefinder"
    />
  );
}
