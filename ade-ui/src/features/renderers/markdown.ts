// From-scratch Markdown parser (M5). No dependencies — a small block tokenizer
// plus an inline parser, producing a token tree that `Markdown.svelte` renders.
//
// Supported: ATX headings, fenced code blocks (``` with info string), blockquotes,
// unordered/ordered lists (nested), tables (GFM pipe), thematic breaks,
// paragraphs; inline: bold, italic, inline code, links, images, strikethrough,
// autolinks, hard breaks, and `$...$` / `$$...$$` math spans. Deliberately a
// pragmatic common subset — not a full CommonMark implementation.

export type Inline =
  | { type: "text"; value: string }
  | { type: "strong"; children: Inline[] }
  | { type: "em"; children: Inline[] }
  | { type: "del"; children: Inline[] }
  | { type: "code"; value: string }
  | { type: "math"; value: string } // inline $...$
  | { type: "link"; href: string; children: Inline[] }
  | { type: "image"; src: string; alt: string }
  | { type: "break" };

export type Block =
  | { type: "heading"; level: number; children: Inline[] }
  | { type: "paragraph"; children: Inline[] }
  | { type: "code"; lang: string; value: string }
  | { type: "mathBlock"; value: string } // $$...$$
  | { type: "blockquote"; children: Block[] }
  | { type: "list"; ordered: boolean; start: number; items: Block[][] }
  | { type: "table"; header: Inline[][]; align: Align[]; rows: Inline[][][] }
  | { type: "hr" };

export type Align = "left" | "center" | "right" | null;

// ---------------------------------------------------------------------------
// Block tokenizer
// ---------------------------------------------------------------------------

export function parseMarkdown(src: string): Block[] {
  // Normalize newlines and tabs; keep it simple.
  const lines = src.replace(/\r\n?/g, "\n").replace(/\t/g, "    ").split("\n");
  return parseBlocks(lines);
}

function parseBlocks(lines: string[]): Block[] {
  const blocks: Block[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];

    // Blank line — skip.
    if (line.trim() === "") {
      i++;
      continue;
    }

    // Fenced code block: ``` or ~~~ with optional info string.
    const fence = line.match(/^(\s*)(`{3,}|~{3,})\s*([^`]*)$/);
    if (fence) {
      const marker = fence[2][0];
      const len = fence[2].length;
      const lang = fence[3].trim();
      const body: string[] = [];
      i++;
      while (i < lines.length) {
        const close = lines[i].match(/^(\s*)(`{3,}|~{3,})\s*$/);
        if (close && close[2][0] === marker && close[2].length >= len) {
          i++;
          break;
        }
        body.push(lines[i]);
        i++;
      }
      blocks.push({ type: "code", lang, value: body.join("\n") });
      continue;
    }

    // $$ ... $$ display math (own lines).
    if (line.trim() === "$$") {
      const body: string[] = [];
      i++;
      while (i < lines.length && lines[i].trim() !== "$$") {
        body.push(lines[i]);
        i++;
      }
      i++; // consume closing $$
      blocks.push({ type: "mathBlock", value: body.join("\n") });
      continue;
    }
    // $$...$$ on a single line.
    const inlineDisplay = line.match(/^\s*\$\$(.+?)\$\$\s*$/);
    if (inlineDisplay) {
      blocks.push({ type: "mathBlock", value: inlineDisplay[1].trim() });
      i++;
      continue;
    }

    // ATX heading.
    const heading = line.match(/^(#{1,6})\s+(.*?)\s*#*\s*$/);
    if (heading) {
      blocks.push({
        type: "heading",
        level: heading[1].length,
        children: parseInline(heading[2]),
      });
      i++;
      continue;
    }

    // Thematic break.
    if (/^\s*([-*_])(\s*\1){2,}\s*$/.test(line)) {
      blocks.push({ type: "hr" });
      i++;
      continue;
    }

    // Blockquote.
    if (/^\s*>/.test(line)) {
      const quoted: string[] = [];
      while (i < lines.length && /^\s*>/.test(lines[i])) {
        quoted.push(lines[i].replace(/^\s*>\s?/, ""));
        i++;
      }
      blocks.push({ type: "blockquote", children: parseBlocks(quoted) });
      continue;
    }

    // Table (GFM): a header row followed by a delimiter row of ---|:--:.
    if (line.includes("|") && i + 1 < lines.length && isTableDelimiter(lines[i + 1])) {
      const header = splitRow(line).map(parseInline);
      const align = splitRow(lines[i + 1]).map(cellAlign);
      i += 2;
      const rows: Inline[][][] = [];
      while (i < lines.length && lines[i].includes("|") && lines[i].trim() !== "") {
        rows.push(splitRow(lines[i]).map(parseInline));
        i++;
      }
      blocks.push({ type: "table", header, align, rows });
      continue;
    }

    // List (ordered or unordered), possibly nested by indent.
    if (/^(\s*)([-*+]|\d+[.)])\s+/.test(line)) {
      const [list, next] = parseList(lines, i);
      blocks.push(list);
      i = next;
      continue;
    }

    // Paragraph: gather until a blank line or a block-starting line.
    const para: string[] = [];
    while (i < lines.length && lines[i].trim() !== "" && !startsBlock(lines[i])) {
      para.push(lines[i]);
      i++;
    }
    blocks.push({ type: "paragraph", children: parseInline(para.join("\n")) });
  }

  return blocks;
}

// Whether a line begins a non-paragraph block (so paragraphs stop before it).
function startsBlock(line: string): boolean {
  return (
    /^\s*(`{3,}|~{3,})/.test(line) ||
    /^#{1,6}\s/.test(line) ||
    /^\s*>/.test(line) ||
    /^\s*([-*+]|\d+[.)])\s+/.test(line) ||
    /^\s*([-*_])(\s*\1){2,}\s*$/.test(line) ||
    line.trim() === "$$"
  );
}

function isTableDelimiter(line: string): boolean {
  return /^\s*\|?\s*:?-{1,}:?\s*(\|\s*:?-{1,}:?\s*)*\|?\s*$/.test(line) && line.includes("-");
}

function splitRow(line: string): string[] {
  let s = line.trim();
  s = s.replace(/^\|/, "").replace(/\|$/, "");
  // Split on unescaped pipes.
  const cells: string[] = [];
  let cur = "";
  for (let j = 0; j < s.length; j++) {
    if (s[j] === "\\" && s[j + 1] === "|") {
      cur += "|";
      j++;
    } else if (s[j] === "|") {
      cells.push(cur.trim());
      cur = "";
    } else {
      cur += s[j];
    }
  }
  cells.push(cur.trim());
  return cells;
}

function cellAlign(cell: string): Align {
  const c = cell.trim();
  const l = c.startsWith(":");
  const r = c.endsWith(":");
  if (l && r) return "center";
  if (r) return "right";
  if (l) return "left";
  return null;
}

// Parse a (possibly nested) list starting at `start`. Returns [list, nextIndex].
function parseList(lines: string[], start: number): [Block, number] {
  const first = lines[start].match(/^(\s*)([-*+]|\d+[.)])\s+/)!;
  const baseIndent = first[1].length;
  const ordered = /\d/.test(first[2]);
  const startNum = ordered ? parseInt(first[2], 10) : 1;
  const items: Block[][] = [];
  let i = start;

  while (i < lines.length) {
    const m = lines[i].match(/^(\s*)([-*+]|\d+[.)])\s+(.*)$/);
    // A sibling must match indent *and* list kind (ordered vs unordered), so a
    // `1.` list after a `-` list starts a fresh block.
    if (!m || m[1].length !== baseIndent || /\d/.test(m[2]) !== ordered) {
      // Not a sibling item at this level.
      if (lines[i].trim() === "") {
        // Allow a single blank line inside the list.
        if (i + 1 < lines.length && lines[i + 1].match(/^(\s*)([-*+]|\d+[.)])\s+/)) {
          i++;
          continue;
        }
      }
      break;
    }

    // Collect this item's lines: the marker line's content plus any following
    // lines indented deeper than the marker.
    const itemLines: string[] = [m[3]];
    i++;
    while (i < lines.length) {
      if (lines[i].trim() === "") {
        // Peek: continues only if the next non-blank is indented under the item.
        const nextNonBlank = lines[i + 1] ?? "";
        if (/^\s{2,}\S/.test(nextNonBlank) && nextNonBlank.search(/\S/) > baseIndent) {
          itemLines.push("");
          i++;
          continue;
        }
        break;
      }
      const indent = lines[i].search(/\S/);
      const isSibling = lines[i].match(/^(\s*)([-*+]|\d+[.)])\s+/);
      if (isSibling && (isSibling[1].length ?? 0) <= baseIndent) break;
      if (indent > baseIndent) {
        // Deeper-indented content (nested list or continuation) belongs to this
        // item. De-indent by the base indent + marker width (~2) so the nested
        // parse sees it at column 0.
        itemLines.push(lines[i].slice(Math.min(indent, baseIndent + 2)));
        i++;
      } else {
        break;
      }
    }
    items.push(parseBlocks(itemLines));
  }

  return [{ type: "list", ordered, start: startNum, items }, i];
}

// ---------------------------------------------------------------------------
// Inline parser
// ---------------------------------------------------------------------------

export function parseInline(src: string): Inline[] {
  const out: Inline[] = [];
  let i = 0;
  let text = "";

  const flush = () => {
    if (text) {
      out.push({ type: "text", value: text });
      text = "";
    }
  };

  while (i < src.length) {
    const c = src[i];
    const rest = src.slice(i);

    // Hard break: two+ trailing spaces before newline, or backslash-newline.
    if ((c === " " && /^ {2,}\n/.test(rest)) || (c === "\\" && src[i + 1] === "\n")) {
      flush();
      out.push({ type: "break" });
      i += rest.match(/^ *\n/)?.[0].length ?? 2;
      continue;
    }
    // Collapse a lone newline to a space (soft break).
    if (c === "\n") {
      text += " ";
      i++;
      continue;
    }

    // Escaped punctuation.
    if (c === "\\" && /[\\`*_{}\[\]()#+\-.!~$]/.test(src[i + 1] ?? "")) {
      text += src[i + 1];
      i += 2;
      continue;
    }

    // Inline code: `code` (longest matching backtick run).
    if (c === "`") {
      const run = rest.match(/^`+/)![0];
      const close = src.indexOf(run, i + run.length);
      if (close !== -1) {
        flush();
        out.push({ type: "code", value: src.slice(i + run.length, close).trim() });
        i = close + run.length;
        continue;
      }
    }

    // Inline math: $...$ (not $$, no space right after opening $).
    if (c === "$" && src[i + 1] !== "$") {
      const m = rest.match(/^\$([^$\n]+?)\$/);
      if (m && !/^\s/.test(m[1])) {
        flush();
        out.push({ type: "math", value: m[1] });
        i += m[0].length;
        continue;
      }
    }

    // Image: ![alt](src)
    if (c === "!" && src[i + 1] === "[") {
      const m = rest.match(/^!\[([^\]]*)\]\(([^)\s]+)(?:\s+"[^"]*")?\)/);
      if (m) {
        flush();
        out.push({ type: "image", alt: m[1], src: m[2] });
        i += m[0].length;
        continue;
      }
    }

    // Link: [text](href)
    if (c === "[") {
      const m = rest.match(/^\[([^\]]*)\]\(([^)\s]+)(?:\s+"[^"]*")?\)/);
      if (m) {
        flush();
        out.push({ type: "link", href: m[2], children: parseInline(m[1]) });
        i += m[0].length;
        continue;
      }
    }

    // Autolink: <https://...>
    if (c === "<") {
      const m = rest.match(/^<((?:https?|mailto):[^>\s]+)>/);
      if (m) {
        flush();
        out.push({ type: "link", href: m[1], children: [{ type: "text", value: m[1] }] });
        i += m[0].length;
        continue;
      }
    }

    // Strong: ** or __
    if ((c === "*" || c === "_") && src[i + 1] === c) {
      const marker = c + c;
      const close = findClose(src, i + 2, marker);
      if (close !== -1) {
        flush();
        out.push({ type: "strong", children: parseInline(src.slice(i + 2, close)) });
        i = close + 2;
        continue;
      }
    }

    // Strikethrough: ~~
    if (c === "~" && src[i + 1] === "~") {
      const close = findClose(src, i + 2, "~~");
      if (close !== -1) {
        flush();
        out.push({ type: "del", children: parseInline(src.slice(i + 2, close)) });
        i = close + 2;
        continue;
      }
    }

    // Emphasis: single * or _
    if (c === "*" || c === "_") {
      const close = findClose(src, i + 1, c);
      if (close !== -1 && close > i + 1) {
        flush();
        out.push({ type: "em", children: parseInline(src.slice(i + 1, close)) });
        i = close + 1;
        continue;
      }
    }

    text += c;
    i++;
  }

  flush();
  return out;
}

// Find the next unescaped occurrence of `marker` from `from`.
function findClose(src: string, from: number, marker: string): number {
  let i = from;
  while (i < src.length) {
    if (src[i] === "\\") {
      i += 2;
      continue;
    }
    if (src.startsWith(marker, i)) return i;
    i++;
  }
  return -1;
}
