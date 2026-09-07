# BRIEF — 296 J: every type wat uses is declared in wat

> Read `DESIGN-STONE-J-every-wat-type-is-declared-in-wat.md` first. H-3 is the worked example:
> `wat/core.wat:2125` + `src/types.rs`'s `wat_enum_register_from!` calls, landed green today.

## THE WORK

Move all **25** enum registrations out of `src/types.rs` into `.wat` declarations, then **close the
door behind them** with a lint that refuses a hand-written `TypeDef::Enum(EnumDef { … })` literal.
Both halves, one strike.

## READ IN ORDER

```
src/types.rs                  the 25 literals. READ THE PROSE ABOVE EACH — purity is an argued
                              DECISION, and several carry rationale that must travel with them.
                              1247 doc lines total; they are the point, not overhead.
wat/core.wat:2120-2131        H-3's landed pair — the exact shape to copy, prose included
src/types.rs (H-3's calls)    wat_enum_register_from!(env, "<file>", ":wat::ns::Name")
wat/kernel/diagnostics.wat    a family type-home file, for the shape of a new one
src/load/stdlib.rs            STDLIB_FILES — every NEW .wat file joins it, in a valid position
tests/lint/wat_record_from_sources_are_loaded.rs   the gate that catches a missed load-set entry,
                              and the model for the new wall (read its "why the obvious test is
                              VACUOUS" section before writing yours)
crates/wat-source-derive/src/lib.rs:95  binder_vector — the crate's ONE `:-` recogniser (H-3)
```

## HOMES — the design's table, applied

```
wat/core.wat             ReadOutcome · ReadWithCommentsOutcome                    (exists)
wat/holon.wat            CombineOutcome · CosineOutcome · DegenerateSide ·
                         DotOutcome · VectorDecodeOutcome                          (exists)
wat/kernel/outcomes.wat  Accept · Close · Connect · ReadFrame · Readln · Recv ·
                         RunResult · Send · Signal · SignalOutcome · TrySend       (NEW)
wat/edn.wat              ReadForeignOutcome · ReadJsonOutcome · Validation         (NEW)
wat/eval.wat             FormOutcome · StepResult · WalkStep                       (NEW)
wat/stream.wat           NextOutcome                                              (NEW)
```

## THE WALL

A lint over `src/types.rs`: a `register_builtin(TypeDef::Enum(EnumDef { … }))` **literal** is
refused; the only admitted form is `wat_enum_register_from!`. Model it on
`wat_record_from_sources_are_loaded.rs` — including its discipline of stating what the obvious
version of the test would fail to catch. It goes red on all 25 until the sweep lands, which is why
the sweep comes first **in this same strike**.

## STOP TRIGGERS

- **STOP-1 — a variant/field/purity changes.** This is a MOVE. Every declaration must be
  byte-equivalent to the literal it replaces or the stdlib refuses to load (`Existing::Equivalent`).
  A load failure means the transcription is wrong; fix it, never widen the gate.
- **STOP-2 — prose is dropped.** The 1247 doc lines are half the reason for the stone. A variant
  arriving in wat without the sentence that explained it in Rust is a silent loss. If a comment
  genuinely describes Rust-side mechanics rather than the type, say so per case in the SCORE.
- **STOP-3 — a new file cannot find a load-set position without a cycle.** Report it; do not
  reorder `STDLIB_FILES` to force it. The order is a contract with its own header.
- **STOP-4 — a type turns out to be UNREACHABLE from wat** (`CloseOutcome`, `DegenerateSide` are
  the suspects — zero textual uses). That is a `purgare` FINDING, not a reason to skip it. Declare
  it and record the finding; retirement is a separate ruling.
- **STOP-5 — the wall needs an exemption to land.** If any of the 25 genuinely cannot be
  wat-sourced, STOP and report which and why. An `#[allow]` or a rune on the wall's first day means
  the wall was drawn wrong.
- **STOP-6 — a record or struct literal drags in.** Out of scope, affirmatively: enums only, so a
  red is never ambiguous between the two classes.

## PRIOR ART

`SCORE-296-H3-option-and-result-live-in-wat.md` — the same move at n=2, including the parametric
registration this stone's 9 parametric types depend on.
