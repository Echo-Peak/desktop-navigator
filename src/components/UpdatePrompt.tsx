import * as React from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { invoke, isTauri } from "@/lib/tauri";

export function UpdatePrompt() {
  const [version, setVersion] = React.useState<string | null>(null);

  React.useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | null = null;
    (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<{ version: string }>("update:ready", (e) => {
        setVersion(e.payload.version);
      });
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const restartNow = () => {
    invoke("update_restart").catch(() => undefined);
  };

  return (
    <Dialog open={version !== null} onOpenChange={(o) => !o && setVersion(null)}>
      <DialogContent className="w-[420px]">
        <DialogTitle>Update ready</DialogTitle>
        <DialogDescription>
          Version {version} has been installed. Restart to apply it now, or it
          will take effect the next time you open the app.
        </DialogDescription>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={() => setVersion(null)}>
            Restart later
          </Button>
          <Button size="sm" onClick={restartNow}>
            Restart now
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
