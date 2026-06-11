import { Handle, Position, type NodeProps } from "@xyflow/react";
import { ACTION_ICONS } from "@/lib/actions";
import type { AutomationStep } from "@/types/page-context";
import { ACTION_CATALOG } from "@/types/action-catalog";

export type ActionNodeData = { step: AutomationStep };

function labelFor(type: string): string {
  return ACTION_CATALOG.find((a) => a.type === type)?.label ?? type;
}

export function ActionNode({ data }: NodeProps) {
  const step = (data as ActionNodeData).step;
  const Icon = ACTION_ICONS[step.action.type];
  return (
    <div className="flex min-w-44 items-center gap-3 rounded-xl border border-border bg-panel-2 px-3 py-2.5 shadow">
      <Handle type="target" position={Position.Top} className="!bg-accent" />
      <span className="flex h-8 w-8 items-center justify-center rounded-lg bg-accent/15 text-accent">
        <Icon size={16} />
      </span>
      <div className="min-w-0">
        <div className="text-sm font-medium">{labelFor(step.action.type)}</div>
        {step.description && (
          <div className="truncate text-xs text-muted">{step.description}</div>
        )}
      </div>
      <Handle type="source" position={Position.Bottom} className="!bg-accent" />
    </div>
  );
}
