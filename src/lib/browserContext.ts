const KEY = "dn:browser-config";

export interface BrowserContextJson {
  schemaVersion: string;
  browserConfig: {
    incognito: boolean;
    hadAudio: boolean;
    isFullscreen: boolean;
    executablePath?: string;
    width: number;
    height: number;
    xPosition: number;
    yPosition: number;
    flags?: string[];
  };
}

export function buildBrowserContext(): BrowserContextJson {
  let cfg: Record<string, unknown> = {};
  try {
    cfg = JSON.parse(localStorage.getItem(KEY) || "{}");
  } catch {
    cfg = {};
  }
  const flags = String(cfg.flags ?? "")
    .split(/\s+/)
    .filter(Boolean);
  return {
    schemaVersion: "1.0",
    browserConfig: {
      incognito: Boolean(cfg.incognito),
      hadAudio: false,
      isFullscreen: Boolean(cfg.isFullscreen),
      executablePath: cfg.executablePath ? String(cfg.executablePath) : undefined,
      width: Number(cfg.width ?? 1280),
      height: Number(cfg.height ?? 800),
      xPosition: Number(cfg.xPosition ?? 0),
      yPosition: Number(cfg.yPosition ?? 0),
      flags: flags.length ? flags : undefined
    }
  };
}
