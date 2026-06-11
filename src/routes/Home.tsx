import * as React from "react";
import { useNavigate } from "react-router-dom";
import { Plus, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger
} from "@/components/ui/dialog";
import {
  deletePage,
  faviconUrl,
  listPages,
  pageName,
  readPage,
  writePage
} from "@/lib/storage";
import type { PageContext } from "@/types";

interface Entry {
  name: string;
  page: PageContext;
}

export function Home() {
  const navigate = useNavigate();
  const [entries, setEntries] = React.useState<Entry[]>([]);

  const refresh = React.useCallback(async () => {
    const names = await listPages();
    const loaded = await Promise.all(
      names.map(async (name) => ({ name, page: await readPage(name) }))
    );
    setEntries(
      loaded.filter((e): e is Entry => e.page !== null)
    );
  }, []);

  React.useEffect(() => {
    refresh();
  }, [refresh]);

  const remove = async (name: string) => {
    await deletePage(name);
    refresh();
  };

  return (
    <section>
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-xl font-semibold">Automations</h1>
          <p className="text-sm text-muted">
            One PageContext per domain. Open a card to build its automation.
          </p>
        </div>
        <CreatePageDialog
          onCreated={(name) => navigate(`/automations?page=${name}`)}
        />
      </div>

      {entries.length === 0 ? (
        <div className="rounded-xl border border-dashed border-border p-12 text-center text-muted">
          No automations yet. Create one to get started.
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-4">
          {entries.map(({ name, page }) => (
            <Card
              key={name}
              className="group cursor-pointer transition-colors hover:border-accent"
              onClick={() => navigate(`/automations?page=${name}`)}
            >
              <div className="flex items-start justify-between">
                <img
                  src={faviconUrl(page.domain)}
                  alt=""
                  className="h-8 w-8 rounded"
                />
                <button
                  className="text-muted opacity-0 transition-opacity hover:text-danger group-hover:opacity-100"
                  onClick={(e) => {
                    e.stopPropagation();
                    remove(name);
                  }}
                >
                  <Trash2 size={16} />
                </button>
              </div>
              <CardTitle className="mt-3">{page.domain}</CardTitle>
              <CardDescription>
                {page.description || "No description"}
              </CardDescription>
              <div className="mt-3 text-xs text-muted">
                {page.steps.length} step{page.steps.length === 1 ? "" : "s"}
              </div>
            </Card>
          ))}
        </div>
      )}
    </section>
  );
}

function CreatePageDialog({
  onCreated
}: {
  onCreated: (name: string) => void;
}) {
  const [open, setOpen] = React.useState(false);
  const [domain, setDomain] = React.useState("");
  const [description, setDescription] = React.useState("");

  const create = async () => {
    if (!domain.trim()) return;
    const page: PageContext = {
      domain: domain.trim(),
      description: description.trim(),
      schemaVersion: "1.0",
      env: {},
      steps: [],
      canvas: { nodes: [], edges: [] }
    };
    await writePage(page);
    setOpen(false);
    setDomain("");
    setDescription("");
    onCreated(pageName(page.domain));
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm">
          <Plus size={14} /> New automation
        </Button>
      </DialogTrigger>
      <DialogContent className="w-[420px]">
        <DialogTitle>New automation</DialogTitle>
        <div className="mt-4 space-y-3">
          <Field label="Domain">
            <input
              autoFocus
              value={domain}
              onChange={(e) => setDomain(e.target.value)}
              placeholder="example.com"
              className="w-full rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent"
            />
          </Field>
          <Field label="Description">
            <input
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="What does this automate?"
              className="w-full rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent"
            />
          </Field>
        </div>
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="ghost" size="sm" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button size="sm" onClick={create}>
            Create
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function Field({
  label,
  children
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs text-muted">{label}</span>
      {children}
    </label>
  );
}
