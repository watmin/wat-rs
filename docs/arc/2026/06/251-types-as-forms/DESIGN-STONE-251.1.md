# DESIGN — Stone 251.1 — LIFT + WARD `src/resolve/`, then normalize symbol refs

**Parent:** arc 251 (`DESIGN.md`) + 251.0 (`DESIGN-STONE-251.0.md`, mechanism =
normalize-layer). **Status:** strike drawn; split into 251.1a/b/c.

251.1 lifts the surface→entity resolution into a warded home and teaches it to
resolve dotted-symbol refs. Per the lift-and-ward direction: surgical, scoped,
bar-raised, relentless — no stone leaves its segment flat.

---

## The split (four-questions verdict B — recorded)

A single bundled stone (lift + normalize + consolidate) FAILS Simple — three
concerns, three failure modes, one muddy "did it work." The stepping-stone split
gives each stone a single verification axis (recovery-doc § Proactive slicing):

- **251.1a — LIFT (pure move, green→green).** `src/resolve.rs` (709 lines, flat,
  unstamped) → `src/resolve/` directory home. NO behavior change. Re-exports keep
  `crate::resolve::{resolve_references, ResolveError, UnresolvedReference,
  is_reserved_prefix}` (the lib.rs:155 + freeze.rs:61 importers) working unchanged.
  intueri names the internal modules. Vigilia 6-8 spell → L1+L2=0; vigilatum stamp.
  Verification: lib + corpus baseline identical pre/post; clippy-clean in-home.
  *Why first:* 251.1b's transform then lands in already-warded ground — smaller
  cognitive surface, the behavior change isolated from the structural move.

- **251.1b — NORMALIZE (the one behavior change).** Add the validate **+ normalize**
  transform: a `WatAST::Symbol` ref whose name carries a namespace segment
  (`wat.core/+` — discriminated from a bare local `x` by the `<ns>/<name>` shape)
  resolves to the entity its keyword FQDN names, and the node is rewritten to canonical
  so downstream dispatch (untouched) resolves it. This lifts resolve's current
  "does NOT transform the AST" limitation (module doc) — a deliberate, named shift.
  Verification: `probe_arc251_stone0_symbol_head` C01 RED→GREEN, C02 stays GREEN
  (dual-read); lib + corpus baseline green. Re-earn the stamp.

- **251.1c — CONSOLIDATE (behavior-preserving migration).** Move `check.rs:1637`
  `BARE_PRIMITIVES` / `BARE_CONTAINER_HEADS` (+ the :1753/:1770 application sites) into
  the resolve home, so resolution is the single surface→entity canonicalization
  authority (bare→FQDN AND symbol→entity in one place). Verification: lib + corpus
  identical; no behavior drift. Re-earn the stamp.

---

## The carve (251.1a — provisional; intueri finalizes the module names)

`src/resolve.rs` today holds: the error types (`ResolveError`,
`UnresolvedReference`), the `use!`/rust-deps declaration collection, the call-head
walk (`check_form`, `is_resolvable_call_head`), and the quote-family boundary descent
(`check_quasiquote_template`, `matches?` boundary). Provisional split (names are
intueri's call, per `feedback_intueri_names_all_things`):

```
src/resolve/
  mod.rs        — home doc + vigilatum stamp + re-exports (the public surface
                  lib.rs/freeze.rs import); resolve_references entry
  error.rs      — ResolveError, UnresolvedReference
  <walk>.rs     — check_form / is_resolvable_call_head / the call-head walk
  <quote>.rs    — quasiquote/quote/forms/matches? boundary descent
  <rust_use>.rs — collect_use_declarations + the :rust::* use! coverage check
  (251.1b adds) <normalize>.rs — symbol-ref → entity transform
  (251.1c adds) <canonical>.rs — BARE_PRIMITIVES / BARE_CONTAINER_HEADS
```

The exact module boundaries + names are intueri's to set — the cast grounds them on
the `src/function/`, `src/check/`, `src/types/` home precedents (error.rs is the
settled name for the error module across homes).

## One contract decision pinned

> 251.1a preserves the EXACT public surface (`crate::resolve::*` re-exports) and
> behavior — it is a structural lift, nothing more. Any behavior delta in 251.1a is a
> bug, not a feature. The transform arrives only in 251.1b.

## Out of scope = rejected

- The symbol transform — 251.1b, not 1a.
- The BARE-table consolidation — 251.1c, not 1a.
- `wat.type/` namespace / parametrics / `:-` / HARD-CUT — later stones (251.2-251.5).
- Native symbol-AST head representation — types get genuine forms at 251.3; 251.1 is
  normalize-layer only (per 251.0).

## Next (251.1a)

1. Crawl resolve.rs fully (the tail past line 260 — the boundary helpers).
2. Spawn **intueri** (verbatim spell, embed-never-fetch) to name the `src/resolve/`
   internal module structure; ground on the home precedents.
3. BRIEF + EXPECTATIONS for the pure lift (positive-only; the carve as read-in-order
   rooms; STOP triggers = rejection criteria).
4. Spawn sonnet (`model:"sonnet"`, background); time-box.
5. Score: lib + corpus baseline identical pre/post; clippy-clean in-home; the probe
   unchanged (still C01 RED — 251.1a doesn't touch behavior). Then vigilia ward → stamp.
