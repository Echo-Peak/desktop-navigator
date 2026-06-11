import * as React from "react";
import { Save } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card } from "@/components/ui/card";

const KEY = "dn:browser-config";

interface BrowserConfig {
  executablePath: string;
  flags: string;
  isFullscreen: boolean;
  incognito: boolean;
  width: number;
  height: number;
  xPosition: number;
  yPosition: number;
}

const DEFAULTS: BrowserConfig = {
  executablePath: "",
  flags: "",
  isFullscreen: false,
  incognito: false,
  width: 1280,
  height: 800,
  xPosition: 0,
  yPosition: 0
};

function load(): BrowserConfig {
  try {
    return { ...DEFAULTS, ...JSON.parse(localStorage.getItem(KEY) || "{}") };
  } catch {
    return DEFAULTS;
  }
}

export function BrowserConfig() {
  const [cfg, setCfg] = React.useState<BrowserConfig>(DEFAULTS);
  const [saved, setSaved] = React.useState(false);

  React.useEffect(() => setCfg(load()), []);

  const set = <K extends keyof BrowserConfig>(k: K, v: BrowserConfig[K]) =>
    setCfg((c) => ({ ...c, [k]: v }));

  const save = () => {
    localStorage.setItem(KEY, JSON.stringify(cfg));
    setSaved(true);
    window.setTimeout(() => setSaved(false), 1500);
  };

  return (
    <section className="max-w-2xl">
      <h1 className="mb-1 text-xl font-semibold">Browser Config</h1>
      <p className="mb-6 text-sm text-muted">
        Settings used when launching the managed browser session.
      </p>

      <Card className="space-y-4">
        <Text
          label="Executable path"
          placeholder="Leave empty to auto-detect Chromium"
          value={cfg.executablePath}
          onChange={(v) => set("executablePath", v)}
        />
        <Text
          label="Extra flags (space-separated)"
          placeholder="--lang=en-US"
          value={cfg.flags}
          onChange={(v) => set("flags", v)}
        />
        <div className="grid grid-cols-2 gap-4">
          <Num label="Width" value={cfg.width} onChange={(v) => set("width", v)} />
          <Num label="Height" value={cfg.height} onChange={(v) => set("height", v)} />
          <Num label="X position" value={cfg.xPosition} onChange={(v) => set("xPosition", v)} />
          <Num label="Y position" value={cfg.yPosition} onChange={(v) => set("yPosition", v)} />
        </div>
        <Check
          label="Maximized / fullscreen"
          checked={cfg.isFullscreen}
          onChange={(v) => set("isFullscreen", v)}
        />
        <Check
          label="Incognito"
          checked={cfg.incognito}
          onChange={(v) => set("incognito", v)}
        />
        <div className="flex justify-end">
          <Button size="sm" onClick={save}>
            <Save size={14} /> {saved ? "Saved" : "Save"}
          </Button>
        </div>
      </Card>
    </section>
  );
}

const inputCls =
  "w-full rounded-lg border border-border bg-panel-2 px-3 py-2 text-sm outline-none focus:border-accent";

function Text({
  label,
  value,
  onChange,
  placeholder
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs text-muted">{label}</span>
      <input
        className={inputCls}
        value={value}
        placeholder={placeholder}
        onChange={(e) => onChange(e.target.value)}
      />
    </label>
  );
}

function Num({
  label,
  value,
  onChange
}: {
  label: string;
  value: number;
  onChange: (v: number) => void;
}) {
  return (
    <label className="block">
      <span className="mb-1 block text-xs text-muted">{label}</span>
      <input
        type="number"
        className={inputCls}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
      />
    </label>
  );
}

function Check({
  label,
  checked,
  onChange
}: {
  label: string;
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <label className="flex items-center gap-2 text-sm">
      <input
        type="checkbox"
        checked={checked}
        onChange={(e) => onChange(e.target.checked)}
      />
      {label}
    </label>
  );
}
