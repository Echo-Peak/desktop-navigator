import { Plus } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ACTION_CATALOG, type ActionType } from "@/types/action-catalog";
import { ACTION_ICONS } from "@/lib/actions";

const COMMANDS = ACTION_CATALOG.filter(
  (a) => a.category === "os" || a.category === "browser"
);
const DOM_ACTIONS = ACTION_CATALOG.filter((a) => a.category === "dom-data");

export function ExpandablePanel({
  onAdd
}: {
  onAdd: (type: ActionType) => void;
}) {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm">
          <Plus size={14} /> Add action
        </Button>
      </DialogTrigger>
      <DialogContent className="h-[480px] w-[560px] overflow-hidden">
        <DialogTitle>Actions</DialogTitle>
        <Tabs defaultValue="dom" className="mt-3 flex h-[400px] flex-col">
          <TabsList>
            <TabsTrigger value="dom">DOM Actions</TabsTrigger>
            <TabsTrigger value="commands">Commands</TabsTrigger>
          </TabsList>
          <TabsContent value="dom" className="mt-3 flex-1 overflow-y-auto pr-1">
            <Grid items={DOM_ACTIONS} onAdd={onAdd} />
          </TabsContent>
          <TabsContent
            value="commands"
            className="mt-3 flex-1 overflow-y-auto pr-1"
          >
            <Grid items={COMMANDS} onAdd={onAdd} />
          </TabsContent>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
}

function Grid({
  items,
  onAdd
}: {
  items: typeof ACTION_CATALOG;
  onAdd: (type: ActionType) => void;
}) {
  return (
    <div className="grid grid-cols-6 gap-2">
      {items.map((a) => {
        const Icon = ACTION_ICONS[a.type];
        return (
          <button
            key={a.type}
            draggable
            onDragStart={(e) =>
              e.dataTransfer.setData("application/dn-action", a.type)
            }
            onClick={() => onAdd(a.type)}
            className="flex flex-col items-center gap-1.5 rounded-lg border border-border bg-panel-2 p-2 text-center transition-colors hover:border-accent"
          >
            <Icon size={18} className="text-accent" />
            <span className="text-[11px] leading-tight text-muted">
              {a.label}
            </span>
          </button>
        );
      })}
    </div>
  );
}
