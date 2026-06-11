import { AnimatePresence, motion } from "framer-motion";
import { Square } from "lucide-react";
import { useSession } from "@/state/session";

export function SessionOverlay() {
  const { active, cycle, stop } = useSession();
  return (
    <AnimatePresence>
      {active && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="pointer-events-none fixed inset-0 z-[100]"
        >
          <div className="absolute inset-0 bg-black/60" />
          <motion.div
            initial={{ y: 24, opacity: 0 }}
            animate={{ y: 0, opacity: 1 }}
            exit={{ y: 24, opacity: 0 }}
            className="pointer-events-auto absolute bottom-8 left-1/2 flex -translate-x-1/2 items-center gap-4 rounded-full border border-border bg-panel/90 px-5 py-3 backdrop-blur"
          >
            <span className="flex items-center gap-2 text-sm">
              <span className="h-2 w-2 animate-pulse rounded-full bg-danger" />
              Running <strong>{cycle}</strong>
            </span>
            <button
              onClick={stop}
              className="flex items-center gap-2 rounded-full bg-danger px-3 py-1.5 text-sm font-medium text-white"
            >
              <Square size={14} /> Stop (Esc)
            </button>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}
