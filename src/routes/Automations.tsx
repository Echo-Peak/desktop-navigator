import * as React from "react";
import { useSearchParams } from "react-router-dom";
import { Circle, Play, Save, Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogTitle
} from "@/components/ui/dialog";
import { FlowCanvas, type CanvasHandle } from "@/components/canvas/FlowCanvas";
import { ExpandablePanel } from "@/components/canvas/ExpandablePanel";
import { readPage, writePage } from "@/lib/storage";
import { runPage } from "@/lib/runner";
import { invoke, isTauri } from "@/lib/tauri";
import { buildBrowserContext } from "@/lib/browserContext";
import {
  eventToStep,
  eventsToSteps,
  hasSensitive,
  stepsToCanvas
} from "@/lib/recorder";
import { useSession } from "@/state/session";
import type { ActionType } from "@/types/action-catalog";
import type { PageContext } from "@/types";
import type { RecordedEvent } from "@/types/recorder";

type RecorderEnvelope =
  | { kind: "ready"; sessionId: string }
  | { kind: "event"; payload: RecordedEvent }
  | { kind: "closed" };

function startUrl(domain: string): string {
  return domain.startsWith("http") ? domain : `https://${domain}`;
}

function formatElapsed(ms: number): string {
  const total = Math.floor(ms / 1000);
  const m = String(Math.floor(total / 60)).padStart(2, "0");
  const s = String(total % 60).padStart(2, "0");
  return `${m}:${s}`;
}

export function Automations() {
  const [params] = useSearchParams();
  const pageParam = params.get("page");
  const [page, setPage] = React.useState<PageContext | null>(null);
  const [loaded, setLoaded] = React.useState(false);
  const [saved, setSaved] = React.useState(false);
  const canvasRef = React.useRef<CanvasHandle>(null);
  const session = useSession();

  const [recording, setRecording] = React.useState(false);
  const [showSave, setShowSave] = React.useState(false);
  const [credPrompt, setCredPrompt] = React.useState(false);
  const [eventCount, setEventCount] = React.useState(0);
  const [elapsed, setElapsed] = React.useState(0);
  const eventsRef = React.useRef<RecordedEvent[]>([]);
  const startedAt = React.useRef(0);

  React.useEffect(() => {
    let cancelled = false;
    (async () => {
      const p = pageParam ? await readPage(pageParam) : null;
      if (!cancelled) {
        setPage(p);
        setLoaded(true);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [pageParam]);

  React.useEffect(() => {
    if (!recording) return;
    const id = window.setInterval(
      () => setElapsed(Date.now() - startedAt.current),
      200
    );
    return () => window.clearInterval(id);
  }, [recording]);

  React.useEffect(() => {
    if (!recording || !isTauri()) return;
    let unlisten: (() => void) | null = null;
    (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<RecorderEnvelope>("recorder:event", (e) => {
        const env = e.payload;
        if (env.kind === "event") {
          eventsRef.current.push(env.payload);
          setEventCount(eventsRef.current.length);
          const step = eventToStep(env.payload);
          if (step) canvasRef.current?.appendStep(step);
        } else if (env.kind === "closed") {
          stopRecording();
        }
      });
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, [recording]);

  const startRecording = async () => {
    if (!page) return;
    eventsRef.current = [];
    setEventCount(0);
    setShowSave(false);
    canvasRef.current?.reset();
    startedAt.current = Date.now();
    setElapsed(0);
    if (!isTauri()) {
      setRecording(true);
      return;
    }
    try {
      await invoke("start_recording", {
        browserContextJson: JSON.stringify(buildBrowserContext()),
        url: startUrl(page.domain)
      });
      setRecording(true);
    } catch (e) {
      session.fail(e instanceof Error ? e.message : String(e));
    }
  };

  const stopRecording = async () => {
    if (isTauri()) await invoke("stop_recording").catch(() => undefined);
    setRecording(false);
    if (eventsRef.current.length > 0) {
      setShowSave(true);
      if (hasSensitive(eventsRef.current)) setCredPrompt(true);
    }
  };

  const saveRecorded = async (mode: "overwrite" | "append") => {
    if (!page) return;
    const recordedSteps = eventsToSteps(eventsRef.current);
    const steps =
      mode === "append" ? [...page.steps, ...recordedSteps] : recordedSteps;
    const canvas = stepsToCanvas(steps);
    const updated = { ...page, steps, canvas };
    await writePage(updated);
    setPage(updated);
    setShowSave(false);
  };

  const save = async () => {
    if (!page || !canvasRef.current) return;
    const { steps, canvas } = canvasRef.current.serialize();
    const updated = { ...page, steps, canvas };
    await writePage(updated);
    setPage(updated);
    setSaved(true);
    window.setTimeout(() => setSaved(false), 1500);
  };

  const addAction = (type: ActionType) => canvasRef.current?.addAction(type);

  const test = async () => {
    if (!page || !canvasRef.current) return;
    const { steps, canvas } = canvasRef.current.serialize();
    session.start();
    try {
      await runPage({ ...page, steps, canvas });
      if (!isTauri()) session.stop();
    } catch (e) {
      session.fail(e instanceof Error ? e.message : String(e));
    }
  };

  if (!loaded) return null;

  if (!page) {
    return (
      <div className="rounded-xl border border-dashed border-border p-12 text-center text-muted">
        Select an automation from Home, or create a new one.
      </div>
    );
  }

  return (
    <section className="flex h-full flex-col">
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold">{page.domain}</h1>
          <p className="text-sm text-muted">{page.description}</p>
        </div>
        <div className="flex items-center gap-2">
          <ExpandablePanel onAdd={addAction} />
          <Button variant="outline" size="sm" onClick={test}>
            <Play size={14} /> Test
          </Button>
          <Button size="sm" onClick={save}>
            <Save size={14} /> {saved ? "Saved" : "Save"}
          </Button>
        </div>
      </div>

      <Tabs defaultValue="recorder" className="flex min-h-0 flex-1 flex-col">
        <div className="flex items-center justify-between">
          <TabsList>
            <TabsTrigger value="recorder">Recorder</TabsTrigger>
            <TabsTrigger value="manual">Manual</TabsTrigger>
          </TabsList>
          <TabsContent value="recorder" className="m-0">
            <div className="flex items-center gap-3">
              {recording && (
                <span className="flex items-center gap-2 text-sm">
                  <span className="h-2 w-2 animate-pulse rounded-full bg-danger" />
                  {formatElapsed(elapsed)} · {eventCount} events
                </span>
              )}
              {recording ? (
                <Button variant="danger" size="sm" onClick={stopRecording}>
                  <Square size={14} /> Stop recording
                </Button>
              ) : (
                <Button size="sm" onClick={startRecording}>
                  <Circle size={14} /> {page.steps.length ? "Re-record" : "Record"}
                </Button>
              )}
            </div>
          </TabsContent>
          <TabsContent value="manual" className="m-0">
            <span className="text-sm text-muted">
              Drag actions from “Add action”, or click to append.
            </span>
          </TabsContent>
        </div>

        <div className="mt-3 min-h-0 flex-1 overflow-hidden rounded-xl border border-border">
          <FlowCanvas key={pageParam ?? "none"} ref={canvasRef} page={page} />
        </div>
      </Tabs>

      <SavePanel
        open={showSave}
        hasExisting={page.steps.length > 0}
        onOverwrite={() => saveRecorded("overwrite")}
        onAppend={() => saveRecorded("append")}
        onDiscard={() => setShowSave(false)}
      />

      <CredentialPrompt
        open={credPrompt}
        onClose={() => setCredPrompt(false)}
      />
    </section>
  );
}

function SavePanel({
  open,
  hasExisting,
  onOverwrite,
  onAppend,
  onDiscard
}: {
  open: boolean;
  hasExisting: boolean;
  onOverwrite: () => void;
  onAppend: () => void;
  onDiscard: () => void;
}) {
  return (
    <Dialog open={open} onOpenChange={(o) => !o && onDiscard()}>
      <DialogContent className="w-[440px]">
        <DialogTitle>Recording finished</DialogTitle>
        <DialogDescription>
          Save the recorded actions as this automation's steps.
          {hasExisting && " Saving as steps overwrites the existing steps."}
        </DialogDescription>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={onDiscard}>
            Discard
          </Button>
          {hasExisting && (
            <Button variant="outline" size="sm" onClick={onAppend}>
              Append to steps
            </Button>
          )}
          <Button size="sm" onClick={onOverwrite}>
            Save as steps
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function CredentialPrompt({
  open,
  onClose
}: {
  open: boolean;
  onClose: () => void;
}) {
  const [value, setValue] = React.useState("");

  const save = async () => {
    if (value && isTauri()) {
      await invoke("store_secret", {
        key: "credentials.password",
        value
      }).catch(() => undefined);
    }
    setValue("");
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(o) => !o && onClose()}>
      <DialogContent className="w-[440px]">
        <DialogTitle>Password field detected</DialogTitle>
        <DialogDescription>
          Save this credential to your OS keychain so the automation can use it
          securely. It is stored as a reference, never in plaintext.
        </DialogDescription>
        <input
          type="password"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          placeholder="Password"
          className="mt-4 w-full rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent"
        />
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={onClose}>
            Skip
          </Button>
          <Button size="sm" onClick={save}>
            Save to keychain
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
