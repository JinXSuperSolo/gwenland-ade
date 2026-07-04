// From-scratch TeX-math → HTML renderer (M5). A pragmatic common subset — not
// full KaTeX. Handles: sub/superscripts, \frac, \sqrt, grouping `{}`, Greek and
// common symbol macros, \text, \left/\right delimiters, \sum/\int with limits,
// and operators. Unknown macros degrade to their literal name.
//
// Output is a string of HTML using spans + a little CSS (see Katex.svelte).
// Everything is escaped; no raw user HTML is emitted.

const GREEK: Record<string, string> = {
  alpha: "α", beta: "β", gamma: "γ", delta: "δ", epsilon: "ε", varepsilon: "ε",
  zeta: "ζ", eta: "η", theta: "θ", vartheta: "ϑ", iota: "ι", kappa: "κ",
  lambda: "λ", mu: "μ", nu: "ν", xi: "ξ", pi: "π", varpi: "ϖ", rho: "ρ",
  varrho: "ϱ", sigma: "σ", varsigma: "ς", tau: "τ", upsilon: "υ", phi: "φ",
  varphi: "ϕ", chi: "χ", psi: "ψ", omega: "ω",
  Gamma: "Γ", Delta: "Δ", Theta: "Θ", Lambda: "Λ", Xi: "Ξ", Pi: "Π",
  Sigma: "Σ", Upsilon: "Υ", Phi: "Φ", Psi: "Ψ", Omega: "Ω",
};

const SYMBOLS: Record<string, string> = {
  times: "×", div: "÷", pm: "±", mp: "∓", cdot: "·", ast: "∗", star: "⋆",
  leq: "≤", le: "≤", geq: "≥", ge: "≥", neq: "≠", ne: "≠", approx: "≈",
  equiv: "≡", cong: "≅", sim: "∼", propto: "∝", ll: "≪", gg: "≫",
  infty: "∞", partial: "∂", nabla: "∇", forall: "∀", exists: "∃",
  in: "∈", notin: "∉", subset: "⊂", supset: "⊃", subseteq: "⊆", supseteq: "⊇",
  cup: "∪", cap: "∩", emptyset: "∅", varnothing: "∅",
  rightarrow: "→", to: "→", leftarrow: "←", leftrightarrow: "↔",
  Rightarrow: "⇒", Leftarrow: "⇐", Leftrightarrow: "⇔", mapsto: "↦",
  land: "∧", wedge: "∧", lor: "∨", vee: "∨", neg: "¬", oplus: "⊕", otimes: "⊗",
  angle: "∠", perp: "⊥", parallel: "∥", cdots: "⋯", ldots: "…", dots: "…",
  vdots: "⋮", ddots: "⋱", prime: "′", circ: "∘", bullet: "•", degree: "°",
  hbar: "ℏ", ell: "ℓ", Re: "ℜ", Im: "ℑ", aleph: "ℵ", nabla2: "∇",
  sum: "∑", prod: "∏", int: "∫", oint: "∮", iint: "∬", bigcup: "⋃", bigcap: "⋂",
  lfloor: "⌊", rfloor: "⌋", lceil: "⌈", rceil: "⌉", langle: "⟨", rangle: "⟩",
};

// Named operators rendered upright (\sin, \log, …).
const OPS = new Set([
  "sin", "cos", "tan", "cot", "sec", "csc", "arcsin", "arccos", "arctan",
  "sinh", "cosh", "tanh", "log", "ln", "lg", "exp", "lim", "max", "min",
  "sup", "inf", "det", "dim", "ker", "deg", "gcd", "arg", "mod",
]);

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

type Tok =
  | { t: "char"; v: string }
  | { t: "macro"; v: string }
  | { t: "open" } // {
  | { t: "close" } // }
  | { t: "sup" } // ^
  | { t: "sub" } // _
  | { t: "amp" }; // & (unused outside matrices, treated as space)

function tokenize(src: string): Tok[] {
  const toks: Tok[] = [];
  let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (c === "\\") {
      const m = src.slice(i).match(/^\\([a-zA-Z]+|.)/);
      if (m) {
        toks.push({ t: "macro", v: m[1] });
        i += m[0].length;
        continue;
      }
      i++;
      continue;
    }
    if (c === "{") { toks.push({ t: "open" }); i++; continue; }
    if (c === "}") { toks.push({ t: "close" }); i++; continue; }
    if (c === "^") { toks.push({ t: "sup" }); i++; continue; }
    if (c === "_") { toks.push({ t: "sub" }); i++; continue; }
    if (c === "&") { toks.push({ t: "amp" }); i++; continue; }
    if (c === " ") { i++; continue; } // TeX ignores spaces
    toks.push({ t: "char", v: c });
    i++;
  }
  return toks;
}

// Recursive-descent over the token stream. Returns HTML.
function render(toks: Tok[]): string {
  let i = 0;

  // Parse a single "atom" (a group, macro application, or char) → HTML.
  function atom(): string {
    const tok = toks[i];
    if (!tok) return "";

    if (tok.t === "open") {
      i++;
      let html = "";
      while (i < toks.length && toks[i].t !== "close") html += withScripts(atom());
      if (toks[i]?.t === "close") i++;
      return html;
    }

    if (tok.t === "macro") {
      i++;
      return macro(tok.v);
    }

    if (tok.t === "char") {
      i++;
      // Italicize single letters (variables), keep digits/punct upright.
      if (/[a-zA-Z]/.test(tok.v)) return `<i>${esc(tok.v)}</i>`;
      return esc(tok.v);
    }

    if (tok.t === "amp") { i++; return "<span class='k-sp'></span>"; }
    // Stray sup/sub/close — consume to avoid a loop.
    i++;
    return "";
  }

  // After an atom, attach any ^{} / _{} scripts.
  function withScripts(base: string): string {
    let sup = "";
    let sub = "";
    while (toks[i]?.t === "sup" || toks[i]?.t === "sub") {
      const kind = toks[i].t;
      i++;
      const s = atom();
      if (kind === "sup") sup = s;
      else sub = s;
    }
    if (sup && sub) {
      return `${base}<span class="k-scripts"><sup>${sup}</sup><sub>${sub}</sub></span>`;
    }
    if (sup) return `${base}<sup>${sup}</sup>`;
    if (sub) return `${base}<sub>${sub}</sub>`;
    return base;
  }

  function macro(name: string): string {
    // \frac{a}{b}
    if (name === "frac" || name === "dfrac" || name === "tfrac") {
      const num = atom();
      const den = atom();
      return `<span class="k-frac"><span class="k-num">${num}</span><span class="k-den">${den}</span></span>`;
    }
    // \sqrt[n]{x}
    if (name === "sqrt") {
      let index = "";
      const isChar = (t: Tok | undefined, v: string) => t?.t === "char" && t.v === v;
      if (isChar(toks[i], "[")) {
        i++;
        let idx = "";
        while (i < toks.length && !isChar(toks[i], "]")) idx += atom();
        if (toks[i]?.t === "char") i++; // consume ]
        index = `<span class="k-root-idx">${idx}</span>`;
      }
      const rad = atom();
      return `<span class="k-sqrt">${index}<span class="k-radical">√</span><span class="k-under">${rad}</span></span>`;
    }
    // \text{...} and friends — upright.
    if (name === "text" || name === "mathrm" || name === "operatorname" || name === "mathbf" || name === "mathsf") {
      const inner = rawGroup();
      const bold = name === "mathbf";
      return `<span class="k-text"${bold ? ' style="font-weight:600"' : ""}>${esc(inner)}</span>`;
    }
    // \left( ... \right) delimiters — render the delimiters literally.
    if (name === "left") {
      const d = delimiter();
      return `<span class="k-delim">${d}</span>`;
    }
    if (name === "right") {
      const d = delimiter();
      return `<span class="k-delim">${d}</span>`;
    }
    if (name === "hat" || name === "bar" || name === "vec" || name === "tilde" || name === "dot") {
      const acc = { hat: "^", bar: "‾", vec: "→", tilde: "~", dot: "˙" }[name]!;
      const base = atom();
      return `<span class="k-acc"><span class="k-acc-mark">${acc}</span>${base}</span>`;
    }
    if (name === "boxed") {
      return `<span class="k-boxed">${atom()}</span>`;
    }
    // Named operators upright.
    if (OPS.has(name)) return `<span class="k-op">${name}</span>`;
    // Greek + symbols.
    if (GREEK[name]) return `<span class="k-sym">${GREEK[name]}</span>`;
    if (SYMBOLS[name]) return `<span class="k-sym">${SYMBOLS[name]}</span>`;
    if (name === "quad") return "<span class='k-quad'></span>";
    if (name === ",") return "<span class='k-thin'></span>";
    if (name === "\\") return "<br/>";
    // Unknown macro → literal name.
    return esc(name);
  }

  // Read a `{...}` group as raw text (for \text).
  function rawGroup(): string {
    let s = "";
    if (toks[i]?.t === "open") {
      i++;
      let depth = 1;
      while (i < toks.length && depth > 0) {
        const t = toks[i];
        if (t.t === "open") depth++;
        else if (t.t === "close") { depth--; if (depth === 0) { i++; break; } }
        if (depth > 0) {
          if (t.t === "char") s += t.v;
          else if (t.t === "macro") s += t.v;
          else if (t.t === "sup") s += "^";
          else if (t.t === "sub") s += "_";
          i++;
        }
      }
    } else {
      s = atom();
    }
    return s;
  }

  // Read a delimiter char/macro after \left or \right.
  function delimiter(): string {
    const t = toks[i];
    if (!t) return "";
    if (t.t === "char") {
      i++;
      return t.v === "." ? "" : esc(t.v);
    }
    if (t.t === "macro") {
      i++;
      return SYMBOLS[t.v] ?? esc(t.v);
    }
    i++;
    return "";
  }

  let html = "";
  while (i < toks.length) html += withScripts(atom());
  return html;
}

/// Renders a TeX string to HTML. Never throws — on any failure it returns the
/// escaped source so the app degrades gracefully.
export function renderMath(tex: string): string {
  try {
    return render(tokenize(tex));
  } catch {
    return `<span class="k-err">${esc(tex)}</span>`;
  }
}
