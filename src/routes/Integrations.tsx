import * as React from "react";
import { Plus, Trash2, Webhook, FileText } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger
} from "@/components/ui/dialog";
import type { Integration } from "@/types";

const KEY = "dn:integrations";

function load(): Integration[] {
  try {
    return JSON.parse(localStorage.getItem(KEY) || "[]");
  } catch {
    return [];
  }
}

function persist(items: Integration[]) {
  localStorage.setItem(KEY, JSON.stringify(items));
}

export function Integrations() {
  const [items, setItems] = React.useState<Integration[]>([]);

  React.useEffect(() => setItems(load()), []);

  const add = (i: Integration) => {
    const next = [...items, i];
    setItems(next);
    persist(next);
  };

  const remove = (idx: number) => {
    const next = items.filter((_, i) => i !== idx);
    setItems(next);
    persist(next);
  };

  return (
    <section>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold">Integrations</h1>
          <p className="text-sm text-muted">Where automation output is sent.</p>
        </div>
        <AddIntegration onAdd={add} />
      </div>

      {items.length === 0 ? (
        <div className="rounded-xl border border-dashed border-border p-12 text-center text-muted">
          No integrations yet.
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
          {items.map((i, idx) => (
            <Card key={idx} className="group">
              <div className="flex items-start justify-between">
                <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/15 text-accent">
                  {i.type === "webhook" ? (
                    <Webhook size={16} />
                  ) : (
                    <FileText size={16} />
                  )}
                </span>
                <button
                  className="text-muted opacity-0 transition-opacity hover:text-danger group-hover:opacity-100"
                  onClick={() => remove(idx)}
                >
                  <Trash2 size={16} />
                </button>
              </div>
              <CardTitle className="mt-3 capitalize">{i.type}</CardTitle>
              <CardDescription className="truncate">
                {i.destination}
              </CardDescription>
            </Card>
          ))}
        </div>
      )}
    </section>
  );
}

function AddIntegration({ onAdd }: { onAdd: (i: Integration) => void }) {
  const [open, setOpen] = React.useState(false);
  const [type, setType] = React.useState<Integration["type"]>("webhook");
  const [destination, setDestination] = React.useState("");

  const submit = () => {
    if (!destination.trim()) return;
    onAdd({ type, destination: destination.trim(), payloadTemplate: {} });
    setOpen(false);
    setDestination("");
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus size={14} /> New integration
        </Button>
      </DialogTrigger>
      <DialogContent className="w-[420px]">
        <DialogTitle>New integration</DialogTitle>
        <div className="mt-4 space-y-3">
          <label className="block">
            <span className="mb-1 block text-xs text-muted">Type</span>
            <select
              value={type}
              onChange={(e) => setType(e.target.value as Integration["type"])}
              className="w-full rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent"
            >
              <option value="webhook">Webhook</option>
              <option value="file">File</option>
            </select>
          </label>
          <label className="block">
            <span className="mb-1 block text-xs text-muted">
              {type === "webhook" ? "URL" : "File path"}
            </span>
            <input
              value={destination}
              onChange={(e) => setDestination(e.target.value)}
              className="w-full rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent"
            />
          </label>
        </div>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button size="sm" onClick={submit}>
            Add
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}
