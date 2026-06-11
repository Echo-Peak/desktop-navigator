const KEY = "dn:cycles";
const MEMBERS_KEY = "dn:cycle-members";

export const DEFAULT_CYCLE = "Default";

export function listCycles(): string[] {
  try {
    const extra: string[] = JSON.parse(localStorage.getItem(KEY) || "[]");
    return [DEFAULT_CYCLE, ...extra];
  } catch {
    return [DEFAULT_CYCLE];
  }
}

export function addCycle(name: string) {
  const trimmed = name.trim();
  if (!trimmed || trimmed === DEFAULT_CYCLE) return;
  const extra: string[] = JSON.parse(localStorage.getItem(KEY) || "[]");
  localStorage.setItem(KEY, JSON.stringify([...new Set([...extra, trimmed])]));
}

type Members = Record<string, string[]>;

function loadMembers(): Members {
  try {
    return JSON.parse(localStorage.getItem(MEMBERS_KEY) || "{}");
  } catch {
    return {};
  }
}

export function getCycleMembers(cycle: string): string[] {
  return loadMembers()[cycle] ?? [];
}

export function setCycleMembers(cycle: string, pageNames: string[]) {
  const members = loadMembers();
  members[cycle] = pageNames;
  localStorage.setItem(MEMBERS_KEY, JSON.stringify(members));
}

export function resolveCyclePages(
  cycle: string,
  allPageNames: string[]
): string[] {
  if (cycle === DEFAULT_CYCLE) return allPageNames;
  const members = getCycleMembers(cycle);
  return allPageNames.filter((n) => members.includes(n));
}
