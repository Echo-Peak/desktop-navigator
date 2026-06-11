import * as React from "react";
import { Dialog, DialogContent, DialogTitle, DialogDescription } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

const FLAG = "dn:onboarded";

const STEPS = [
  {
    title: "Welcome to Desktop Navigator",
    body: "Automate a real browser by building visual automations — no code required."
  },
  {
    title: "Record or build",
    body: "Use the Recorder to capture clicks and typing, or the Manual tab to drag actions onto the canvas."
  },
  {
    title: "Run a cycle",
    body: "Group automations into cycles and press Play in the top bar. Press Escape any time to stop."
  }
];

export function Onboarding() {
  const [open, setOpen] = React.useState(false);
  const [step, setStep] = React.useState(0);

  React.useEffect(() => {
    if (!localStorage.getItem(FLAG)) setOpen(true);
  }, []);

  const finish = () => {
    localStorage.setItem(FLAG, "1");
    setOpen(false);
  };

  const current = STEPS[step];
  const last = step === STEPS.length - 1;

  return (
    <Dialog open={open} onOpenChange={(o) => !o && finish()}>
      <DialogContent className="w-[440px]">
        <DialogTitle>{current.title}</DialogTitle>
        <DialogDescription>{current.body}</DialogDescription>
        <div className="mt-5 flex items-center justify-between">
          <div className="flex gap-1.5">
            {STEPS.map((_, i) => (
              <span
                key={i}
                className={
                  i === step
                    ? "h-1.5 w-6 rounded-full bg-accent"
                    : "h-1.5 w-1.5 rounded-full bg-border"
                }
              />
            ))}
          </div>
          <div className="flex gap-2">
            <Button variant="ghost" size="sm" onClick={finish}>
              Skip
            </Button>
            <Button
              size="sm"
              onClick={() => (last ? finish() : setStep(step + 1))}
            >
              {last ? "Get started" : "Next"}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
