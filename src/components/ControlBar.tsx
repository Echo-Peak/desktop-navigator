import * as React from "react";
import { ChevronDown, Play, Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger
} from "@/components/ui/dropdown-menu";
import { useSession } from "@/state/session";
import { listCycles } from "@/lib/cycles";
import { runCycle } from "@/lib/runner";
import { isTauri } from "@/lib/tauri";

export function ControlBar() {
  const { active, cycle, error, setCycle, start, stop, fail } = useSession();
  const [cycles, setCycles] = React.useState<string[]>([]);

  React.useEffect(() => {
    setCycles(listCycles());
  }, []);

  const play = async () => {
    start();
    try {
      await runCycle(cycle);
      if (!isTauri()) stop();
    } catch (e) {
      fail(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <header className="flex h-14 items-center justify-between border-b border-border px-6">
      <div className="flex items-center gap-3">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <Button variant="outline" size="sm">
              {cycle}
              <ChevronDown size={14} />
            </Button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="start">
            {cycles.map((c) => (
              <DropdownMenuItem key={c} onSelect={() => setCycle(c)}>
                {c}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>

        {active ? (
          <Button variant="danger" size="sm" onClick={stop}>
            <Square size={14} /> Stop
          </Button>
        ) : (
          <Button size="sm" onClick={play}>
            <Play size={14} /> Play
          </Button>
        )}
      </div>
      {error ? (
        <div className="max-w-md truncate text-sm text-danger" title={error}>
          {error}
        </div>
      ) : (
        <div className="text-sm text-muted">Desktop Navigator</div>
      )}
    </header>
  );
}
