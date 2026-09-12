import { useModuleFront } from "@/lib/Modulefront";
import { LidarView } from "@/lib/Modules/lidar";
import { useEffect } from "react";

export default function LidarPage() {
    useEffect(() => {
        useModuleFront.getState().MakeFakeLidar();
    }, []);
  return (
    
      <LidarView id="fake-lidar" />
    
  );
}