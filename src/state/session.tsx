import * as React from "react";
import { invoke, isTauri } from "@/lib/tauri";

interface SessionState {
  active: boolean;
  cycle: string;
  error: string | null;
  setCycle: (c: string) => void;
  start: () => void;
  stop: () => void;
  fail: (message: string) => void;
}

const SessionContext = React.createContext<SessionState | null>(null);

export function SessionProvider({ children }: { children: React.ReactNode }) {
  const [active, setActive] = React.useState(false);
  const [cycle, setCycle] = React.useState("Default");
  const [error, setError] = React.useState<string | null>(null);

  const stop = React.useCallback(() => {
    setActive(false);
    if (isTauri()) {
      invoke("end_session").catch(() => undefined);
    }
  }, []);

  const start = React.useCallback(() => {
    setError(null);
    setActive(true);
  }, []);

  const fail = React.useCallback(
    (message: string) => {
      setError(message);
      stop();
    },
    [stop]
  );

  React.useEffect(() => {
    if (!active) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") stop();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [active, stop]);

  React.useEffect(() => {
    if (!isTauri()) return;
    let unlisten: Array<() => void> = [];
    (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten.push(
        await listen("session:done", () => {
          setActive(false);
        })
      );
      unlisten.push(
        await listen<string>("session:error", (e) => {
          setError(e.payload);
          setActive(false);
        })
      );
    })();
    return () => unlisten.forEach((u) => u());
  }, []);

  return (
    <SessionContext.Provider
      value={{ active, cycle, error, setCycle, start, stop, fail }}
    >
      {children}
    </SessionContext.Provider>
  );
}

export function useSession(): SessionState {
  const ctx = React.useContext(SessionContext);
  if (!ctx) throw new Error("useSession must be used within SessionProvider");
  return ctx;
}
