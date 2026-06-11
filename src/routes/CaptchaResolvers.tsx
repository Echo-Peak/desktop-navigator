import * as React from "react";
import { ShieldCheck } from "lucide-react";
import { Card, CardDescription, CardTitle } from "@/components/ui/card";
import type { CaptchaResolver } from "@/types";

const BUILTIN: Pick<CaptchaResolver, "id" | "title" | "description">[] = [
  {
    id: "builtin.checkbox",
    title: "Checkbox Click",
    description: "Detects and clicks a simple verification checkbox."
  },
  {
    id: "builtin.vision",
    title: "Vision Locate",
    description: "Uses the LLM vision module to locate and solve image prompts."
  }
];

const KEY = "dn:captcha-resolvers";

function loadUser(): CaptchaResolver[] {
  try {
    return JSON.parse(localStorage.getItem(KEY) || "[]");
  } catch {
    return [];
  }
}

export function CaptchaResolvers() {
  const [user, setUser] = React.useState<CaptchaResolver[]>([]);

  React.useEffect(() => setUser(loadUser()), []);

  return (
    <section>
      <h1 className="mb-1 text-xl font-semibold">Captcha Resolvers</h1>
      <p className="mb-6 text-sm text-muted">
        Built-in and custom resolvers available to automations.
      </p>

      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 xl:grid-cols-3">
        {[...BUILTIN, ...user].map((r) => (
          <Card key={r.id}>
            <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/15 text-accent">
              <ShieldCheck size={16} />
            </span>
            <CardTitle className="mt-3">{r.title}</CardTitle>
            {r.description && <CardDescription>{r.description}</CardDescription>}
          </Card>
        ))}
      </div>
    </section>
  );
}
