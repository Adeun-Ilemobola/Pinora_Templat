import type { ReactNode } from "react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
export function ModuleCard({
  type,
  id,
  children,
}: {
  type: string;
  id: string;
  children: ReactNode;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle>{type}</CardTitle>
        <CardDescription className="break-all font-mono text-xs">
          ID · {id}
        </CardDescription>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}
