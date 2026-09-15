import { LogViewer } from "@/components/LogViewer";

export default function LogsPage() {
  return (
    <main className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Logs</h1>
        <p className="mt-1 text-sm text-muted-foreground">Registration, system messages, and module activity. Times show receipt on this computer.</p>
      </div>
      <LogViewer />
    </main>
  );
}
