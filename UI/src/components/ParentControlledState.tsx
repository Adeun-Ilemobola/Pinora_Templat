import { LockKeyhole } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";

export function ParentControlledState() {
  return (
    <Alert role="note" className="border-border bg-muted/30 p-3">
      <LockKeyhole
        aria-hidden="true"
        className="size-4 text-muted-foreground"
      />
      <AlertTitle>
        <Badge variant="secondary">Parent controlled</Badge>
      </AlertTitle>
      <AlertDescription className="mt-1">
        Manual control is handled by the parent module. Live readings remain
        available.
      </AlertDescription>
    </Alert>
  );
}
