# Arc 286 — span file-label convention for sourceless eval (STUB, banked)

**Status:** STUB / banked. Not built. Not in the task list (banked arcs live on disk). Low priority —
the substrate is already *correct*; this hardens a convention against a *caller* footgun.

## Trigger (2026-06-19, arc 278 STONE-Value probe)
While capturing the STONE-Value RED-probe HEAD error, a diagnostic rendered `file: "x"`. The `"x"` was NOT
a substrate placeholder — it was a throwaway probe passing `Some("x")` as the file label to
`startup_from_source`. The substrate itself never emits a bare label: when there is no real source file it
uses a clear angle-bracket convention. But the episode surfaced the real gap below.

## The ground truth (what exists, this session)
The sourceless-span file label is a coherent `<…>` convention — but as **scattered ad-hoc string literals**,
not a central definition:
- `Span::unknown()` → `"<runtime>"` (`src/span.rs:71`)
- `startup_from_source(base_canonical = None)` → `"<entry>"` (`src/freeze.rs:722`)
- lexer test / doc convention → `"<test>"` / `"<eval>"` / `"<synthetic>"` (`src/lexer.rs:238,864`)
- frozen fn bodies → `"<fn@{span}>"` (`src/freeze.rs:417,450`)
- plus `"<source>"`, `"<native>"` elsewhere.

## The gap (the failure class)
1. **No single source of truth.** The canonical sourceless labels are duplicated literals across span.rs /
   freeze.rs / lexer.rs. Drift is possible (one path renders `<entry>`, a sibling `<eval>`, for the same
   situation) and there is no one place a reader learns the convention.
2. **No guard on caller-supplied labels.** `file: Arc<String>` accepts *any* string. A caller (test, MCP
   eval, REPL, a future embedder) can inject a bare `"x"`/`""`/`"tmp"` that renders in operator-facing
   diagnostics looking like a real-but-nonexistent file — exactly the confusion that triggered this stub.

## Candidate fix (extirpare ladder — name the rung, don't pre-build)
- **Rung 1 (convention):** document the `<…>` sourceless-label family in one place. Weakest.
- **Rung 2 (central constants):** named constants for the canonical labels (`Span::RUNTIME = "<runtime>"`,
  `ENTRY`, `EVAL`, `SYNTHETIC`, `TEST`); every site references the constant. Drift becomes a compile-time
  rename, not a silent string fork.
- **Rung 3 (top — wrong shape unrepresentable):** make the file label a typed sum, e.g.
  `enum SourceLabel { Sourced(Arc<str>), Runtime, Entry, Eval, Synthetic, Test }`, rendered to the `<…>`
  forms at the diagnostic edge. A bare arbitrary `"x"` is then **uncompilable** — a caller must either name a
  real path (`Sourced`) or pick a declared sourceless variant. This is the honest extirpation (the
  no-fake-correctness discipline: the confusing state has no constructor). Cost: touches the `Span.file`
  type and every construction site — larger; weigh against the megafile-touch constraint (span.rs is small
  and home-able, but `file` is referenced widely).

## Four-questions (sketch — decide at build time)
- **Obvious?** YES — a diagnostic's file label should be a real path or an obviously-synthetic marker, never
  an ambiguous bare token.
- **Simple?** Rung 2 yes; rung 3 is a typed-sum refactor across construction sites — decompose before
  committing.
- **Honest?** Rung 3 is the honest rung (unrepresentable wrong shape); rungs 1–2 lean on discipline.
- **Good UX?** Operator-facing: a clear `<runtime>`/`<eval>` beats `x`. Author-facing: a typed label guides
  callers to the right marker.

## Scope / relations
- Small arc. Likely lands at rung 2 (central constants + doc) unless the typed-sum (rung 3) proves cheap.
- ⚠ **Check overlap with arc 283 (`283-source-file-lift`)** before opening — that arc already touches source
  file handling; this convention may belong inside it rather than as a separate arc.
- NOT a STONE-Value or arc-278 dependency — purely diagnostic hygiene; open any time.
