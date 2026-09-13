import { LidarView } from "@/lib/Modules/lidar";

export default function LidarPage() {
  return (
    <LidarView
      id="lidar"
      servoX_id="servo_x"
      servoY_id="servo_y"
      range_id="rangefinder"
    />
  );
}
