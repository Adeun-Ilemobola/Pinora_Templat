// Local-only browser fixture: exercises real event accumulation and Canvas UI,
// while recording movement locally instead of accessing hardware transport.
import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import { useStore } from "zustand";
import {
  createLidar,
  GenerateGridCells,
  PlayGround,
  type Roi,
} from "../src/lib/Modules/lidar";
import { Button } from "../src/components/ui/button";
import { Switch } from "../src/components/ui/switch";
import "../src/index.css";
const store = createLidar(false, "fixture", "fixture");
const samples = GenerateGridCells().map((cell) => ({
  ...cell.pivotPoint,
  distant: Math.round((cell.gridPoint.x / 180) * 4000),
}));
function Fixture() {
  const state = useStore(store, (s) => s.state);
  const [roi, setRoi] = useState<Roi | null>(null);
  const [mode, setMode] = useState(false);
  const [dark, setDark] = useState(true);
  const [command, setCommand] = useState("None");
  useEffect(() => {
    document.documentElement.classList.toggle("dark", dark);
  }, [dark]);
  return (
    <main className="bg-background p-4 text-foreground">
      <div className="mb-3 flex flex-wrap items-center gap-3">
        <Button
          onClick={() =>
            store
              .getState()
              .handleEvent({
                PointMap: {
                  curr_chunk: 1,
                  max_chunk: 999,
                  map: samples.slice(0, 16380),
                },
              })
          }
        >
          Load first chunk
        </Button>
        <Button
          onClick={() =>
            store
              .getState()
              .handleEvent({
                PointMap: {
                  curr_chunk: 2,
                  max_chunk: 999,
                  map: samples.slice(16380),
                },
              })
          }
        >
          Load second chunk
        </Button>
        <label className="flex items-center gap-2">
          ROI selection
          <Switch checked={mode} onCheckedChange={setMode} />
        </label>
        <Button onClick={() => setDark(!dark)}>Toggle fixture theme</Button>
      </div>
      <output className="block" aria-label="Samples">
        {state.PointMap.map.length} samples
      </output>
      <output className="block" aria-label="Movement">
        {command}
      </output>
      <output className="block" aria-label="Selected ROI">
        {JSON.stringify(roi)}
      </output>
      <PlayGround
        points={state.PointMap.map}
        roi={roi}
        roiMode={mode}
        onRoiChange={setRoi}
        target={null}
        disabled={false}
        onMove={(p) => setCommand(JSON.stringify({ MovePos: { p } }))}
      />
    </main>
  );
}
createRoot(document.getElementById("root")!).render(<Fixture />);
