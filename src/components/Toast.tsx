import * as React from "react";
import { AnimatePresence, motion } from "framer-motion";
import { AlertTriangle, X } from "lucide-react";
import { isTauri } from "@/lib/tauri";

interface ToastItem {
  id: number;
  message: string;
}

export function Toast() {
  const [items, setItems] = React.useState<ToastItem[]>([]);

  const dismiss = (id: number) =>
    setItems((prev) => prev.filter((t) => t.id !== id));

  React.useEffect(() => {
    if (!isTauri()) return;
    let unlisten: (() => void) | null = null;
    (async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<{ message: string }>("update:error", (e) => {
        const id = Date.now();
        setItems((prev) => [...prev, { id, message: e.payload.message }]);
        window.setTimeout(() => dismiss(id), 8000);
      });
    })();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return (
    <div className="pointer-events-none fixed bottom-6 right-6 z-[120] flex flex-col gap-2">
      <AnimatePresence>
        {items.map((t) => (
          <motion.div
            key={t.id}
            initial={{ opacity: 0, x: 24 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: 24 }}
            className="pointer-events-auto flex max-w-sm items-start gap-3 rounded-xl border border-border bg-panel px-4 py-3 shadow-xl"
          >
            <AlertTriangle size={16} className="mt-0.5 shrink-0 text-danger" />
            <span className="text-sm">{t.message}</span>
            <button
              onClick={() => dismiss(t.id)}
              className="ml-auto text-muted hover:text-fg"
            >
              <X size={14} />
            </button>
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  );
}
