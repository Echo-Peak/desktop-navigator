import * as React from "react";
import { Plus, Save } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import {
  addCycle,
  DEFAULT_CYCLE,
  getCycleMembers,
  listCycles,
  setCycleMembers
} from "@/lib/cycles";
import { listPages, readPage } from "@/lib/storage";

const TRIGGER_KEY = "dn:triggers";

interface Triggers {
  idleStartSeconds: number;
  stopOnRightClick: boolean;
  stopOnEscape: boolean;
}

const DEFAULT_TRIGGERS: Triggers = {
  idleStartSeconds: 30,
  stopOnRightClick: true,
  stopOnEscape: true
};

function loadTriggers(): Triggers {
  try {
    return { ...DEFAULT_TRIGGERS, ...JSON.parse(localStorage.getItem(TRIGGER_KEY) || "{}") };
  } catch {
    return DEFAULT_TRIGGERS;
  }
}

interface PageRef {
  name: string;
  domain: string;
}

export function Tasks() {
  const [cycles, setCycles] = React.useState<string[]>([]);
  const [newCycle, setNewCycle] = React.useState("");
  const [triggers, setTriggers] = React.useState<Triggers>(DEFAULT_TRIGGERS);
  const [saved, setSaved] = React.useState(false);
  const [pages, setPages] = React.useState<PageRef[]>([]);

  React.useEffect(() => {
    setCycles(listCycles());
    setTriggers(loadTriggers());
    (async () => {
      const names = await listPages();
      const loaded = await Promise.all(
        names.map(async (name) => {
          const p = await readPage(name);
          return p ? { name, domain: p.domain } : null;
        })
      );
      setPages(loaded.filter((p): p is PageRef => p !== null));
    })();
  }, []);

  const create = () => {
    if (!newCycle.trim()) return;
    addCycle(newCycle);
    setCycles(listCycles());
    setNewCycle("");
  };

  const customCycles = cycles.filter((c) => c !== DEFAULT_CYCLE);

  const saveTriggers = () => {
    localStorage.setItem(TRIGGER_KEY, JSON.stringify(triggers));
    setSaved(true);
    window.setTimeout(() => setSaved(false), 1500);
  };

  return (
    <section className="max-w-3xl">
      <h1 className="mb-1 text-xl font-semibold">Tasks</h1>
      <p className="mb-6 text-sm text-muted">
        Group automations into cycles and configure triggers.
      </p>

      <Card className="mb-6">
        <CardTitle>Cycles</CardTitle>
        <CardDescription>
          Play runs only the selected cycle. “Default” runs every automation.
        </CardDescription>
        <div className="mt-3 flex flex-wrap gap-2">
          {cycles.map((c) => (
            <span
              key={c}
              className="rounded-full border border-border bg-panel-2 px-3 py-1 text-sm"
            >
              {c}
            </span>
          ))}
        </div>
        <div className="mt-4 flex gap-2">
          <input
            value={newCycle}
            onChange={(e) => setNewCycle(e.target.value)}
            placeholder="New cycle name"
            className="flex-1 rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent"
          />
          <Button size="sm" onClick={create}>
            <Plus size={14} /> Add cycle
          </Button>
        </div>
      </Card>

      <Card className="mb-6">
        <CardTitle>Cycle membership</CardTitle>
        <CardDescription>
          Choose which automations belong to each custom cycle. “Default”
          always runs every automation.
        </CardDescription>
        {customCycles.length === 0 ? (
          <p className="mt-3 text-sm text-muted">
            Add a custom cycle above to assign automations.
          </p>
        ) : pages.length === 0 ? (
          <p className="mt-3 text-sm text-muted">No automations to assign yet.</p>
        ) : (
          <div className="mt-4 space-y-5">
            {customCycles.map((cycle) => (
              <MembershipEditor key={cycle} cycle={cycle} pages={pages} />
            ))}
          </div>
        )}
      </Card>

      <Card>
        <CardTitle>Triggers</CardTitle>
        <CardDescription>Start and stop conditions for cycles.</CardDescription>
        <div className="mt-4 space-y-4">
          <label className="block">
            <span className="mb-1 block text-xs text-muted">
              Start on cursor idle (seconds)
            </span>
            <input
              type="number"
              value={triggers.idleStartSeconds}
              onChange={(e) =>
                setTriggers((t) => ({
                  ...t,
                  idleStartSeconds: Number(e.target.value)
                }))
              }
              className="w-40 rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent"
            />
          </label>
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={triggers.stopOnRightClick}
              onChange={(e) =>
                setTriggers((t) => ({ ...t, stopOnRightClick: e.target.checked }))
              }
            />
            Stop cycle on right-click
          </label>
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={triggers.stopOnEscape}
              onChange={(e) =>
                setTriggers((t) => ({ ...t, stopOnEscape: e.target.checked }))
              }
            />
            Stop cycle on Escape
          </label>
          <div className="flex justify-end">
            <Button size="sm" onClick={saveTriggers}>
              <Save size={14} /> {saved ? "Saved" : "Save"}
            </Button>
          </div>
        </div>
      </Card>
    </section>
  );
}

function MembershipEditor({
  cycle,
  pages
}: {
  cycle: string;
  pages: PageRef[];
}) {
  const [members, setMembers] = React.useState<string[]>([]);

  React.useEffect(() => {
    setMembers(getCycleMembers(cycle));
  }, [cycle]);

  const toggle = (name: string, on: boolean) => {
    const next = on
      ? [...new Set([...members, name])]
      : members.filter((m) => m !== name);
    setMembers(next);
    setCycleMembers(cycle, next);
  };

  return (
    <div>
      <div className="mb-2 text-sm font-medium">{cycle}</div>
      <div className="flex flex-wrap gap-3">
        {pages.map((p) => (
          <label key={p.name} className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={members.includes(p.name)}
              onChange={(e) => toggle(p.name, e.target.checked)}
            />
            {p.domain}
          </label>
        ))}
      </div>
    </div>
  );
}
