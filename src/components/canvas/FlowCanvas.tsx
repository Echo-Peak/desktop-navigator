import * as React from "react";
import {
  addEdge,
  Background,
  Controls,
  type Connection,
  type Edge,
  type Node,
  ReactFlow,
  ReactFlowProvider,
  useEdgesState,
  useNodesState,
  useReactFlow
} from "@xyflow/react";
import { ActionNode, type ActionNodeData } from "./ActionNode";
import { defaultAction, newStepId } from "@/lib/actions";
import type { ActionType } from "@/types/action-catalog";
import type {
  AutomationCanvas,
  AutomationStep,
  PageContext
} from "@/types/page-context";

const nodeTypes = { action: ActionNode };

export interface CanvasHandle {
  addAction: (type: ActionType, description?: string) => void;
  appendStep: (step: AutomationStep) => void;
  reset: () => void;
  serialize: () => { steps: AutomationStep[]; canvas: AutomationCanvas };
}

function buildInitial(page: PageContext | null): {
  nodes: Node<ActionNodeData>[];
  edges: Edge[];
} {
  if (!page || page.steps.length === 0) return { nodes: [], edges: [] };
  const posByStep = new Map(
    (page.canvas?.nodes ?? []).map((n) => [n.stepId, n.position])
  );
  const nodes: Node<ActionNodeData>[] = page.steps.map((step, i) => ({
    id: step.id,
    type: "action",
    position: posByStep.get(step.id) ?? { x: 120, y: i * 120 },
    data: { step }
  }));
  const edges: Edge[] =
    page.canvas?.edges && page.canvas.edges.length > 0
      ? page.canvas.edges.map((e) => ({
          id: e.id,
          source: e.source,
          target: e.target
        }))
      : page.steps.slice(1).map((step, i) => ({
          id: `e_${page.steps[i].id}_${step.id}`,
          source: page.steps[i].id,
          target: step.id
        }));
  return { nodes, edges };
}

const Inner = React.forwardRef<CanvasHandle, { page: PageContext | null }>(
  ({ page }, ref) => {
    const initial = React.useMemo(() => buildInitial(page), [page]);
    const [nodes, setNodes, onNodesChange] = useNodesState<Node<ActionNodeData>>(
      initial.nodes
    );
    const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>(initial.edges);
    const { screenToFlowPosition, getViewport, setCenter } = useReactFlow();

    const addAt = React.useCallback(
      (type: ActionType, position: { x: number; y: number }, description?: string) => {
        const step: AutomationStep = {
          id: newStepId(),
          action: defaultAction(type),
          ...(description ? { description } : {})
        };
        setNodes((prev) => {
          const last = prev[prev.length - 1];
          const node: Node<ActionNodeData> = {
            id: step.id,
            type: "action",
            position,
            data: { step }
          };
          if (last) {
            setEdges((es) =>
              addEdge(
                { id: `e_${last.id}_${step.id}`, source: last.id, target: step.id },
                es
              )
            );
          }
          return [...prev, node];
        });
      },
      [setNodes, setEdges]
    );

    const appendStep = React.useCallback(
      (step: AutomationStep) => {
        setNodes((prev) => {
          const last = prev[prev.length - 1];
          const y = prev.length * 120 + 40;
          const node: Node<ActionNodeData> = {
            id: step.id,
            type: "action",
            position: { x: 140, y },
            data: { step }
          };
          if (last) {
            setEdges((es) =>
              addEdge(
                { id: `e_${last.id}_${step.id}`, source: last.id, target: step.id },
                es
              )
            );
          }
          setCenter(140, y, { zoom: 1, duration: 300 });
          return [...prev, node];
        });
      },
      [setNodes, setEdges, setCenter]
    );

    React.useImperativeHandle(ref, () => ({
      addAction: (type, description) =>
        appendStep({
          id: newStepId(),
          action: defaultAction(type),
          ...(description ? { description } : {})
        }),
      appendStep,
      reset: () => {
        setNodes([]);
        setEdges([]);
      },
      serialize: () => ({
        steps: nodes.map((n) => n.data.step),
        canvas: {
          nodes: nodes.map((n) => ({ stepId: n.id, position: n.position })),
          edges: edges.map((e) => ({
            id: e.id,
            source: e.source,
            target: e.target
          })),
          viewport: getViewport()
        }
      })
    }));

    const onConnect = React.useCallback(
      (c: Connection) => setEdges((es) => addEdge(c, es)),
      [setEdges]
    );

    const onDrop = React.useCallback(
      (e: React.DragEvent) => {
        e.preventDefault();
        const type = e.dataTransfer.getData("application/dn-action") as ActionType;
        if (!type) return;
        const position = screenToFlowPosition({ x: e.clientX, y: e.clientY });
        addAt(type, position);
      },
      [screenToFlowPosition, addAt]
    );

    return (
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onDrop={onDrop}
        onDragOver={(e) => {
          e.preventDefault();
          e.dataTransfer.dropEffect = "move";
        }}
        nodeTypes={nodeTypes}
        fitView
        colorMode="dark"
      >
        <Background />
        <Controls />
      </ReactFlow>
    );
  }
);
Inner.displayName = "FlowCanvasInner";

export const FlowCanvas = React.forwardRef<
  CanvasHandle,
  { page: PageContext | null }
>((props, ref) => (
  <ReactFlowProvider>
    <Inner {...props} ref={ref} />
  </ReactFlowProvider>
));
FlowCanvas.displayName = "FlowCanvas";
