import type { PageContext } from "@/types";
import { invoke, isTauri } from "./tauri";

const PAGE_PREFIX = "dn:page:";
const INDEX_KEY = "dn:pages";

export function pageName(domain: string): string {
  return domain
    .toLowerCase()
    .replace(/^https?:\/\//, "")
    .replace(/[^a-z0-9._-]/g, "_");
}

function localIndex(): string[] {
  try {
    return JSON.parse(localStorage.getItem(INDEX_KEY) || "[]");
  } catch {
    return [];
  }
}

function setLocalIndex(names: string[]) {
  localStorage.setItem(INDEX_KEY, JSON.stringify([...new Set(names)].sort()));
}

export async function listPages(): Promise<string[]> {
  if (isTauri()) {
    return invoke<string[]>("list_pages");
  }
  return localIndex();
}

export async function readPage(name: string): Promise<PageContext | null> {
  if (isTauri()) {
    try {
      const json = await invoke<string>("read_page", { name });
      return JSON.parse(json) as PageContext;
    } catch {
      return null;
    }
  }
  const raw = localStorage.getItem(PAGE_PREFIX + name);
  return raw ? (JSON.parse(raw) as PageContext) : null;
}

export async function writePage(page: PageContext): Promise<string> {
  const name = pageName(page.domain);
  const json = JSON.stringify(page, null, 2);
  if (isTauri()) {
    await invoke("write_page", { name, json });
  } else {
    localStorage.setItem(PAGE_PREFIX + name, json);
    setLocalIndex([...localIndex(), name]);
  }
  return name;
}

export async function deletePage(name: string): Promise<void> {
  if (isTauri()) {
    await invoke("delete_page", { name });
  } else {
    localStorage.removeItem(PAGE_PREFIX + name);
    setLocalIndex(localIndex().filter((n) => n !== name));
  }
}

export function faviconUrl(domain: string): string {
  const host = domain.replace(/^https?:\/\//, "").split("/")[0];
  return `https://www.google.com/s2/favicons?domain=${host}&sz=64`;
}
