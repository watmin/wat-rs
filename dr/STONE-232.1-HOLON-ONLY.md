# Design Reference — Stone 232.1 holon-only (SUPERSEDED by arc 234)

Branch preserves sonnet's 2026-05-23 night flight on Stone 232.1 per the
original BRIEF (`:wat::holon::defprotocol` + `:wat::holon::extend-type`;
holon-only dispatch). Work shipped **12/12 PASS** per its BRIEF.

Then the design space expanded mid-flight (arc 234 wat-record hologram
surfaced) and Stone 232.1's scope revised to `:wat::core::*` polymorphic
protocols operating on records + structs + primitives. This branch is
**REFERENCE-ONLY**, NOT a base for further work.

The artifacts were discarded from the main work branch
(`arc-170-gap-j-v5-deadlock-state`) so the next flight starts from a
clean slate. They live here as design substrate.

## What's here

- `wat/holon/defprotocol.wat` (125 lines) — defprotocol macro at old scope
- `wat/holon/extend-type.wat` (129 lines) — extend-type macro at old scope
- `src/stdlib.rs` (+23 lines) — old WatSource registrations
- `tests/probe_arc232_stone1_defprotocol_macros.rs` (173 lines) — old-scope tests
- `docs/arc/2026/05/232-defprotocol-extend-type/SCORE-STONE-232.1.md` — SCORE per old BRIEF

## What's salvageable here

**Some value, not extraordinary value.** Use as reference, not gospel.

Three concrete patterns to lift:

1. **Splice/quasiquote/iteration macro structure** — `~@(let [...] (map (range 0 n) (fn [i] ...)))` pattern for per-N codegen at macro-expand time. The dispatcher/impl generation shape transfers to the revised scope after primitive-name substitution.

2. **Rest-param eval trap workaround** — use `from-wat → Bundle/children → map over (range 0 n)` to iterate a rest-arg WatAST::List. **Direct iteration over rest-params doesn't work** because the substrate substitutes WatAST::List and the evaluator then tries to call the first element as a function. This pattern is the same one defrecord uses for field-list iteration. Universal to any macro consuming rest-args.

3. **`-> :RetType` explicit annotation in extend-type bodies** — verbose-is-honest; the type checker needs the explicit return type. The macro can't look up the protocol's declared return type at expand time without a substrate-level registry (D7 deferral; would be a Rust-side enhancement).

Plus the D7 verdict itself: **method-name validation against protocol declarations defers to runtime** `UnknownFunction` per arc 233 diagnostic richness. Compile-time validation would require a SymbolTable registry (Rust-side change beyond Stone 232.1's pure-macro scope).

## What's NOT here

- `:wat::core::type` polymorphic primitive (arc 234.0; doesn't exist yet)
- Polymorphism over struct + HashMap (old scope was holon-only)
- The hologram model (arc 234 — `Value::wat_record` dual-form)
- `:wat::core::defrecord` (replaces user-facing `:wat::holon::defrecord` in revised scope)
- Keyword-as-accessor fall-through (arc 234.3 absorbs the queued #058/146 follow-up)
- Hash-destructure in let/match (arc 234.4 absorbs the queued #402)

## Reference signal

If you're authoring revised Stone 232.1 (`:wat::core::*` polymorphic) or
arc 234.x stones, this branch may save 10-20% of structural authoring
time on the macro-iteration patterns. **Don't slavishly mirror — the
scope is meaningfully different**:

- Receiver type changes from `:wat::holon::HolonAST` to polymorphic
- Dispatch primitive changes from `:wat::holon::extract-classifier` to `:wat::core::type`
- File location changes from `wat/holon/` to `wat/core/`
- Macro names change from `:wat::holon::*` to `:wat::core::*`
- Method-body field-access in extend-type bodies operates on whichever backend the receiver lives on (no longer assumes HolonAST)

Use `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` and
`docs/arc/2026/05/232-defprotocol-extend-type/DESIGN.md` (forward-corrected) as the **primary source of truth**. This branch
is supplementary.

## Provenance

- Branched from: `dbda9a0` (main work branch at that point — arc 234 DESIGN draft committed)
- Original BRIEF: `docs/arc/2026/05/232-defprotocol-extend-type/BRIEF-STONE-232.1.md` (commit `04d774c`)
- Supersession decision: arc 234 hologram + revised Stone 232.1 scope per dialogue 2026-05-23 night latest → 2026-05-24 early
- Sonnet flight: 2026-05-23 ~22:00 PDT spawn; 12/12 PASS per old BRIEF; never committed to main work branch
- Disposition: preserved here as design reference; main branch returned to clean state
