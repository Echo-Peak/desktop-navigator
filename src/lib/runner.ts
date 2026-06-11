import type { PageContext } from "@/types";
import { invoke, isTauri } from "./tauri";
import { buildBrowserContext } from "./browserContext";
import { listPages, readPage } from "./storage";
import { resolveCyclePages } from "./cycles";

function startUrlFor(page: PageContext): string {
  const d = page.domain;
  return d.startsWith("http") ? d : `https://${d}`;
}

async function dispatch(pages: PageContext[]): Promise<void> {
  if (pages.length === 0) {
    throw new Error("No automations in this cycle.");
  }
  if (!isTauri()) {
    await new Promise((r) => window.setTimeout(r, 600));
    return;
  }
  await invoke("run_session", {
    browserContextJson: JSON.stringify(buildBrowserContext()),
    pagesJson: pages.map((p) => JSON.stringify(p)),
    startUrl: startUrlFor(pages[0])
  });
}

export async function runCycle(cycle: string): Promise<void> {
  const allNames = await listPages();
  const names = resolveCyclePages(cycle, allNames);
  const loaded = await Promise.all(names.map((n) => readPage(n)));
  const pages = loaded.filter((p): p is PageContext => p !== null);
  await dispatch(pages);
}

export async function runPage(page: PageContext): Promise<void> {
  await dispatch([page]);
}
