import { NavLink, useRouteError } from "react-router";
import { TriangleAlert } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { useModuleFront } from "@/lib/Modulefront";

/**
 * Shown in place of a route whose rendering failed (PIN-TPL-006, Phase 1C).
 *
 * Attached to each child route, so the layout (sidebar and header) stays usable.
 * The connection is not touched; navigating to another route retries rendering.
 */
export function RouteError() {
  const error = useRouteError();
  const clearLogs = useModuleFront((s) => s.clearLogs);
  const message = error instanceof Error ? error.message : String(error);

  return (
    <Alert variant="destructive" role="alert">
      <TriangleAlert aria-hidden="true" />
      <AlertTitle>This view failed to render</AlertTitle>
      <AlertDescription className="space-y-3">
        <p className="break-words font-mono text-xs">{message}</p>
        <p>
          The device connection is unaffected. If this happened on the Logs page,
          clearing the log history removes the entry that caused it.
        </p>
        <div className="flex flex-wrap gap-2">
          <Button variant="outline" size="sm" onClick={clearLogs}>
            Clear log history
          </Button>
          <Button variant="outline" size="sm" render={<NavLink to="/" />}>
            Open dashboard
          </Button>
        </div>
      </AlertDescription>
    </Alert>
  );
}
