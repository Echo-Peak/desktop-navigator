import * as React from "react";
import { Check, Sparkles, Server } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger
} from "@/components/ui/dialog";
import { invoke, isTauri } from "@/lib/tauri";

const KEY = "dn:llm";

type ProviderId = "openrouter" | "ollama";

interface LlmState {
  default: ProviderId;
  configured: Record<ProviderId, boolean>;
  ollamaBaseUrl: string;
  openrouterModel: string;
}

const DEFAULT_STATE: LlmState = {
  default: "openrouter",
  configured: { openrouter: false, ollama: false },
  ollamaBaseUrl: "http://localhost:11434",
  openrouterModel: "openrouter/auto"
};

function load(): LlmState {
  try {
    return { ...DEFAULT_STATE, ...JSON.parse(localStorage.getItem(KEY) || "{}") };
  } catch {
    return DEFAULT_STATE;
  }
}

function persist(s: LlmState) {
  localStorage.setItem(KEY, JSON.stringify(s));
}

export function LlmIntegrations() {
  const [state, setState] = React.useState<LlmState>(DEFAULT_STATE);

  React.useEffect(() => setState(load()), []);

  const update = (s: LlmState) => {
    setState(s);
    persist(s);
  };

  return (
    <section>
      <h1 className="mb-1 text-xl font-semibold">LLM Integrations</h1>
      <p className="mb-6 text-sm text-muted">
        Choose which provider the LLM module uses.
      </p>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
        <ProviderCard
          id="openrouter"
          title="OpenRouter"
          icon={<Sparkles size={16} />}
          state={state}
          onChange={update}
        />
        <ProviderCard
          id="ollama"
          title="Ollama (Local)"
          icon={<Server size={16} />}
          state={state}
          onChange={update}
        />
      </div>
    </section>
  );
}

function ProviderCard({
  id,
  title,
  icon,
  state,
  onChange
}: {
  id: ProviderId;
  title: string;
  icon: React.ReactNode;
  state: LlmState;
  onChange: (s: LlmState) => void;
}) {
  const [open, setOpen] = React.useState(false);
  const [field, setField] = React.useState("");
  const isDefault = state.default === id;
  const configured = state.configured[id];

  React.useEffect(() => {
    setField(id === "ollama" ? state.ollamaBaseUrl : state.openrouterModel);
  }, [open, id, state]);

  const saveConfig = async () => {
    if (id === "openrouter" && isTauri()) {
      await invoke("store_secret", {
        key: "llm.openrouter.apiKey",
        value: field
      }).catch(() => undefined);
    }
    onChange({
      ...state,
      default: id,
      configured: { ...state.configured, [id]: true },
      ...(id === "ollama" ? { ollamaBaseUrl: field } : {})
    });
    setOpen(false);
  };

  return (
    <Card>
      <div className="flex items-start justify-between">
        <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/15 text-accent">
          {icon}
        </span>
        {isDefault && (
          <span className="rounded-full bg-accent/15 px-2 py-0.5 text-xs text-accent">
            Default
          </span>
        )}
      </div>
      <CardTitle className="mt-3">{title}</CardTitle>
      <CardDescription className="flex items-center gap-1">
        {configured ? (
          <>
            <Check size={14} className="text-accent" /> Configured
          </>
        ) : (
          "Not configured"
        )}
      </CardDescription>

      <Dialog open={open} onOpenChange={setOpen}>
        <DialogTrigger asChild>
          <Button variant="outline" size="sm" className="mt-4 w-full">
            Configure
          </Button>
        </DialogTrigger>
        <DialogContent className="w-[420px]">
          <DialogTitle>Configure {title}</DialogTitle>
          <label className="mt-4 block">
            <span className="mb-1 block text-xs text-muted">
              {id === "openrouter" ? "API key" : "Base URL"}
            </span>
            <input
              type={id === "openrouter" ? "password" : "text"}
              value={field}
              onChange={(e) => setField(e.target.value)}
              className="w-full rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent"
            />
          </label>
          {id === "openrouter" && (
            <p className="mt-2 text-xs text-muted">
              Model: {state.openrouterModel}
            </p>
          )}
          <div className="mt-5 flex justify-end gap-2">
            <Button variant="ghost" size="sm" onClick={() => setOpen(false)}>
              Cancel
            </Button>
            <Button size="sm" onClick={saveConfig}>
              Save & set default
            </Button>
          </div>
        </DialogContent>
      </Dialog>
    </Card>
  );
}
