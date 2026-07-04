// From-scratch Mermaid renderer (M5) — a pragmatic subset covering the two most
// common diagram types: `flowchart`/`graph` and `sequenceDiagram`. Produces an
// SVG string. Not a full Mermaid implementation (no subgraphs, class/state/gantt,
// styling directives, etc.) — those degrade to a "diagram unsupported" note.
//
// Layout is intentionally simple: flowcharts use a layered (BFS-rank) placement;
// sequence diagrams use evenly-spaced lifelines with stacked messages.

export type MermaidResult = { svg: string } | { error: string };

export function renderMermaid(src: string): MermaidResult {
  const text = src.trim();
  const firstLine = text.split("\n")[0]?.trim() ?? "";

  try {
    if (/^sequenceDiagram/.test(firstLine)) return { svg: renderSequence(text) };
    if (/^(flowchart|graph)\b/.test(firstLine)) return { svg: renderFlowchart(text) };
    return { error: "Unsupported diagram type. Supported: flowchart / graph, sequenceDiagram." };
  } catch (e) {
    return { error: `Could not render diagram: ${String(e)}` };
  }
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

// Palette pulled from CSS custom props at render time via currentColor + vars.
const C = {
  stroke: "var(--md-graph-stroke)",
  nodeFill: "var(--md-graph-node)",
  nodeText: "var(--md-graph-text)",
  edge: "var(--md-graph-edge)",
  accent: "var(--md-graph-accent)",
};

// ---------------------------------------------------------------------------
// Flowchart
// ---------------------------------------------------------------------------

type Shape = "rect" | "round" | "stadium" | "diamond" | "circle";
type FNode = {
  id: string;
  label: string;
  shape: Shape;
  rank?: number;
  x?: number;
  y?: number;
  w: number;
  h: number;
};
type FEdge = { from: string; to: string; label?: string; dashed?: boolean };

function renderFlowchart(text: string): string {
  const lines = text.split("\n").slice(1).map((l) => l.trim()).filter(Boolean);
  // Direction: TD/TB (top-down) or LR (left-right).
  const dirMatch = text.match(/^(?:flowchart|graph)\s+(TD|TB|LR|RL|BT)/m);
  const dir = dirMatch ? dirMatch[1] : "TD";
  const horizontal = dir === "LR" || dir === "RL";

  const nodes = new Map<string, FNode>();
  const edges: FEdge[] = [];

  const ensure = (id: string, label?: string, shape?: Shape) => {
    let n = nodes.get(id);
    if (!n) {
      n = { id, label: label ?? id, shape: shape ?? "rect", w: 0, h: 40 };
      nodes.set(id, n);
    }
    if (label) n.label = label;
    if (shape) n.shape = shape;
    return n;
  };

  // Parse a node token like  A[Label]  B(Round)  C{Diamond}  D((Circle))  E([Stadium]).
  const parseNode = (tok: string): FNode => {
    tok = tok.trim();
    let m: RegExpMatchArray | null;
    if ((m = tok.match(/^(\w[\w-]*)\(\((.+?)\)\)$/))) return ensure(m[1], unquote(m[2]), "circle");
    if ((m = tok.match(/^(\w[\w-]*)\(\[(.+?)\]\)$/))) return ensure(m[1], unquote(m[2]), "stadium");
    if ((m = tok.match(/^(\w[\w-]*)\{(.+?)\}$/))) return ensure(m[1], unquote(m[2]), "diamond");
    if ((m = tok.match(/^(\w[\w-]*)\((.+?)\)$/))) return ensure(m[1], unquote(m[2]), "round");
    if ((m = tok.match(/^(\w[\w-]*)\[(.+?)\]$/))) return ensure(m[1], unquote(m[2]), "rect");
    if ((m = tok.match(/^(\w[\w-]*)$/))) return ensure(m[1]);
    return ensure(tok);
  };

  for (const line of lines) {
    // Edge:  A --> B ,  A -->|label| B ,  A -.-> B ,  A --- B
    const edge = line.match(
      /^(.+?)\s*(-{2,}>|-\.->|-{2,}|={2,}>)\s*(?:\|([^|]*)\|\s*)?(.+)$/,
    );
    if (edge) {
      const from = parseNode(edge[1]);
      const to = parseNode(edge[4]);
      edges.push({
        from: from.id,
        to: to.id,
        label: edge[3]?.trim(),
        dashed: edge[2].includes("."),
      });
      continue;
    }
    // Bare node declaration.
    if (/^\w/.test(line)) parseNode(line);
  }

  // Rank nodes by longest path from roots (BFS layering).
  layer(nodes, edges);

  // Size nodes from label length.
  for (const n of nodes.values()) {
    const textW = Math.max(40, n.label.length * 7.4 + 24);
    n.w = n.shape === "circle" ? Math.max(50, textW) : textW;
    n.h = n.shape === "circle" ? n.w : 40;
  }

  // Place: group by rank, spread across the cross axis.
  const ranks = new Map<number, FNode[]>();
  for (const n of nodes.values()) {
    const r = n.rank ?? 0;
    (ranks.get(r) ?? ranks.set(r, []).get(r)!).push(n);
  }
  const GAP_MAIN = 80;
  const GAP_CROSS = 34;
  const PAD = 20;

  let mainCursor = PAD;
  const rankKeys = [...ranks.keys()].sort((a, b) => a - b);
  const laneWidths: number[] = [];

  for (const r of rankKeys) {
    const group = ranks.get(r)!;
    const laneSize = Math.max(...group.map((n) => (horizontal ? n.h : n.w)));
    laneWidths.push(laneSize);
    let cross = PAD;
    for (const n of group) {
      if (horizontal) {
        n.x = mainCursor;
        n.y = cross;
        cross += n.h + GAP_CROSS;
      } else {
        n.y = mainCursor;
        n.x = cross;
        cross += n.w + GAP_CROSS;
      }
    }
    mainCursor += laneSize + GAP_MAIN;
  }

  // Center each lane on the cross axis.
  const crossExtent = Math.max(
    ...rankKeys.map((r) => {
      const g = ranks.get(r)!;
      return g.reduce((s, n) => s + (horizontal ? n.h : n.w) + GAP_CROSS, 0) - GAP_CROSS;
    }),
  );
  for (const r of rankKeys) {
    const g = ranks.get(r)!;
    const used = g.reduce((s, n) => s + (horizontal ? n.h : n.w) + GAP_CROSS, 0) - GAP_CROSS;
    const offset = (crossExtent - used) / 2;
    for (const n of g) {
      if (horizontal) n.y = (n.y ?? 0) + offset;
      else n.x = (n.x ?? 0) + offset;
    }
  }

  const width = (horizontal ? mainCursor : crossExtent + 2 * PAD) + PAD;
  const height = (horizontal ? crossExtent + 2 * PAD : mainCursor) + PAD;

  // Draw.
  const parts: string[] = [];
  parts.push(defs());

  for (const e of edges) {
    const a = nodes.get(e.from)!;
    const b = nodes.get(e.to)!;
    if (!a || !b) continue;
    const [x1, y1] = anchor(a, b);
    const [x2, y2] = anchor(b, a);
    parts.push(
      `<path d="M${x1},${y1} L${x2},${y2}" fill="none" stroke="${C.edge}" stroke-width="1.5"` +
        `${e.dashed ? ' stroke-dasharray="5 4"' : ""} marker-end="url(#arrow)"/>`,
    );
    if (e.label) {
      const mx = (x1 + x2) / 2;
      const my = (y1 + y2) / 2;
      parts.push(
        `<rect x="${mx - e.label.length * 3.4 - 4}" y="${my - 9}" width="${e.label.length * 6.8 + 8}" height="18" rx="4" fill="var(--md-graph-bg)"/>`,
        `<text x="${mx}" y="${my + 4}" text-anchor="middle" font-size="11" fill="${C.nodeText}">${esc(e.label)}</text>`,
      );
    }
  }

  for (const n of nodes.values()) parts.push(drawNode(n));

  return svgWrap(width, height, parts.join(""));
}

function anchor(from: FNode, to: FNode): [number, number] {
  const cx = (from.x ?? 0) + from.w / 2;
  const cy = (from.y ?? 0) + from.h / 2;
  const tx = (to.x ?? 0) + to.w / 2;
  const ty = (to.y ?? 0) + to.h / 2;
  const dx = tx - cx;
  const dy = ty - cy;
  // Intersect the direction with the node's bounding box.
  const hw = from.w / 2;
  const hh = from.h / 2;
  const scale = Math.min(
    dx !== 0 ? hw / Math.abs(dx) : Infinity,
    dy !== 0 ? hh / Math.abs(dy) : Infinity,
  );
  return [cx + dx * scale, cy + dy * scale];
}

function drawNode(n: FNode): string {
  const x = n.x ?? 0;
  const y = n.y ?? 0;
  const label = `<text x="${x + n.w / 2}" y="${y + n.h / 2 + 4}" text-anchor="middle" font-size="12.5" fill="${C.nodeText}">${esc(n.label)}</text>`;
  const attrs = `fill="${C.nodeFill}" stroke="${C.stroke}" stroke-width="1.5"`;
  switch (n.shape) {
    case "diamond": {
      const cx = x + n.w / 2, cy = y + n.h / 2;
      const pts = `${cx},${y} ${x + n.w},${cy} ${cx},${y + n.h} ${x},${cy}`;
      return `<polygon points="${pts}" ${attrs}/>${label}`;
    }
    case "circle":
      return `<circle cx="${x + n.w / 2}" cy="${y + n.h / 2}" r="${n.w / 2}" ${attrs}/>${label}`;
    case "stadium":
      return `<rect x="${x}" y="${y}" width="${n.w}" height="${n.h}" rx="${n.h / 2}" ${attrs}/>${label}`;
    case "round":
      return `<rect x="${x}" y="${y}" width="${n.w}" height="${n.h}" rx="12" ${attrs}/>${label}`;
    default:
      return `<rect x="${x}" y="${y}" width="${n.w}" height="${n.h}" rx="4" ${attrs}/>${label}`;
  }
}

// Assign each node a rank = longest path from any root.
function layer(nodes: Map<string, FNode>, edges: FEdge[]) {
  const adj = new Map<string, string[]>();
  const indeg = new Map<string, number>();
  for (const id of nodes.keys()) {
    adj.set(id, []);
    indeg.set(id, 0);
  }
  for (const e of edges) {
    if (!nodes.has(e.from) || !nodes.has(e.to)) continue;
    adj.get(e.from)!.push(e.to);
    indeg.set(e.to, (indeg.get(e.to) ?? 0) + 1);
  }
  // Kahn's algorithm assigning ranks; cycles fall back to rank 0.
  const queue = [...nodes.keys()].filter((id) => (indeg.get(id) ?? 0) === 0);
  for (const id of queue) nodes.get(id)!.rank = 0;
  const seen = new Set(queue);
  let head = 0;
  while (head < queue.length) {
    const id = queue[head++];
    const r = nodes.get(id)!.rank ?? 0;
    for (const nb of adj.get(id)!) {
      const nbn = nodes.get(nb)!;
      nbn.rank = Math.max(nbn.rank ?? 0, r + 1);
      const d = (indeg.get(nb) ?? 1) - 1;
      indeg.set(nb, d);
      if (d === 0 && !seen.has(nb)) {
        seen.add(nb);
        queue.push(nb);
      }
    }
  }
  // Any unranked (cycle) nodes: place after their max predecessor.
  for (const n of nodes.values()) if (n.rank == null) n.rank = 0;
}

// ---------------------------------------------------------------------------
// Sequence diagram
// ---------------------------------------------------------------------------

type Msg = { from: string; to: string; label: string; dashed: boolean };

function renderSequence(text: string): string {
  const lines = text.split("\n").slice(1).map((l) => l.trim()).filter(Boolean);
  const order: string[] = [];
  const labels = new Map<string, string>();
  const msgs: Msg[] = [];

  const addActor = (id: string, label?: string) => {
    if (!labels.has(id)) {
      order.push(id);
      labels.set(id, label ?? id);
    } else if (label) labels.set(id, label);
  };

  for (const line of lines) {
    let m: RegExpMatchArray | null;
    if ((m = line.match(/^(participant|actor)\s+(\w+)(?:\s+as\s+(.+))?$/))) {
      addActor(m[2], m[3]?.trim());
      continue;
    }
    // A->>B: text  |  A-->>B: text  |  A->B  |  A-->B
    if ((m = line.match(/^(\w+)\s*(--?>>?)\s*(\w+)\s*:\s*(.*)$/))) {
      addActor(m[1]);
      addActor(m[3]);
      msgs.push({ from: m[1], to: m[3], label: m[4].trim(), dashed: m[2].includes("--") });
    }
  }

  const LANE = 130;
  const TOP = 50;
  const STEP = 44;
  const PAD = 20;
  const width = PAD * 2 + Math.max(1, order.length) * LANE;
  const height = TOP + msgs.length * STEP + 50;

  const xOf = (id: string) => PAD + order.indexOf(id) * LANE + LANE / 2;

  const parts: string[] = [defs()];

  // Lifelines + actor boxes (top).
  for (const id of order) {
    const x = xOf(id);
    const label = labels.get(id)!;
    const bw = Math.max(60, label.length * 8 + 16);
    parts.push(
      `<line x1="${x}" y1="${TOP}" x2="${x}" y2="${height - 30}" stroke="${C.edge}" stroke-width="1" stroke-dasharray="4 4"/>`,
      `<rect x="${x - bw / 2}" y="${TOP - 32}" width="${bw}" height="28" rx="5" fill="${C.nodeFill}" stroke="${C.stroke}" stroke-width="1.5"/>`,
      `<text x="${x}" y="${TOP - 13}" text-anchor="middle" font-size="12.5" fill="${C.nodeText}">${esc(label)}</text>`,
    );
  }

  // Messages.
  msgs.forEach((msg, idx) => {
    const y = TOP + 24 + idx * STEP;
    const x1 = xOf(msg.from);
    const x2 = xOf(msg.to);
    if (x1 === x2) {
      // Self-message: a little loop.
      parts.push(
        `<path d="M${x1},${y} h30 v18 h-30" fill="none" stroke="${C.edge}" stroke-width="1.5"${msg.dashed ? ' stroke-dasharray="5 4"' : ""} marker-end="url(#arrow)"/>`,
        `<text x="${x1 + 36}" y="${y - 4}" font-size="11.5" fill="${C.nodeText}">${esc(msg.label)}</text>`,
      );
    } else {
      const dir = x2 > x1 ? -1 : 1;
      parts.push(
        `<line x1="${x1}" y1="${y}" x2="${x2 + dir * 4}" y2="${y}" stroke="${C.edge}" stroke-width="1.5"${msg.dashed ? ' stroke-dasharray="5 4"' : ""} marker-end="url(#arrow)"/>`,
        `<text x="${(x1 + x2) / 2}" y="${y - 6}" text-anchor="middle" font-size="11.5" fill="${C.nodeText}">${esc(msg.label)}</text>`,
      );
    }
  });

  return svgWrap(width, height, parts.join(""));
}

// ---------------------------------------------------------------------------
// SVG helpers
// ---------------------------------------------------------------------------

function defs(): string {
  return (
    `<defs><marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">` +
    `<path d="M0,0 L10,5 L0,10 z" fill="${C.edge}"/></marker></defs>`
  );
}

function svgWrap(w: number, h: number, body: string): string {
  return (
    `<svg viewBox="0 0 ${Math.ceil(w)} ${Math.ceil(h)}" width="${Math.ceil(w)}" height="${Math.ceil(h)}" ` +
    `xmlns="http://www.w3.org/2000/svg" font-family="var(--font-sans)">${body}</svg>`
  );
}

function unquote(s: string): string {
  return s.replace(/^["']|["']$/g, "").trim();
}
