# BRIEF — close the ceiling set so a fifth variant cannot be forgotten

Three converters route ceiling breaches to matchable outcomes, each ending in
`_ => Err(EvalBreak::Diagnostic(e))`. There is no gap today — I checked all four variants against
all three converters — but a fifth ceiling variant would land in every `_` at once and silently
become a raise, which is the one thing the outcome wall exists to prevent. Make the ceiling set a
closed inner type and match it exhaustively. Read `DESIGN.md` first — its ⚠ says exactly how far the
exhaustiveness goes, and its "out of scope" cuts three shapes.

## Read in order

1. `src/rete/kernel/outcome.rs:88-105` — the fire converter, its two arms and its `_ =>`.
2. `src/rete/kernel/outcome.rs:148-165` and `:200-215` — the insert and compile converters. Note the
   three owned sets are **disjoint**.
3. `src/value/signal.rs` — `RuntimeErrorKind` and the four ceiling variants with their payloads
   (`limit/used/rounds`, `cap/still_deriving`, and the insert and terminate shapes). They differ;
   the inner enum must carry each.
4. `tests/lint/no_ceiling_raise_in_rete.rs:38-52` — `CEILING_VARIANTS` and the wall's header. It
   guards **construction**; you are closing **routing**. It stays.
5. `src/rete/kernel/outcome.rs` header — *"A second converter is the drift this arc pulls out most
   often."*

## Sketch

```rust
pub enum ReteCeiling {
    SessionMemory { limit: usize, used: usize, rounds: usize },
    FixpointRoundCap { cap: usize, still_deriving: usize },
    SessionMemoryOnInsert { … },
    RuleSetMayNotTerminate { … },
}
// RuntimeErrorKind::ReteCeiling(ReteCeiling)

// a converter, outer `_` kept, inner exhaustive and stated per variant:
Err(EvalBreak::Diagnostic(e)) => match e.kind() {
    RuntimeErrorKind::ReteCeiling(c) => match c {
        ReteCeiling::SessionMemory { .. } => Ok(memory_ceiling_exceeded(..)),
        ReteCeiling::FixpointRoundCap { .. } => Ok(round_cap_exceeded(..)),
        // not this converter's: raise, and say so
        ReteCeiling::SessionMemoryOnInsert { .. } | ReteCeiling::RuleSetMayNotTerminate { .. } =>
            Err(EvalBreak::Diagnostic(e)),
    },
    _ => Err(EvalBreak::Diagnostic(e)),
}
```

## Blast radius

`signal.rs`, `outcome.rs`, and the construction/match sites — **36 references across 7 files** by my
count. Count them yourself and report the number you find.

## Traps named in advance — each with its step

1. **★ Exhaustive over the INNER enum only.** The outer `_ =>` stays — `RuntimeErrorKind` has
   hundreds of variants. **Step:** if you find yourself enumerating non-ceiling kinds, stop.
2. **The cross-converter arms must be WRITTEN, not defaulted.** An insert breach reaching the fire
   converter still raises — but as a named arm. **Step:** no `_` inside the `ReteCeiling` match.
3. **The wat-facing outcome shapes must not move.** These convert *into* `FireOutcome` /
   `InsertOutcome` / `CompileOutcome`, which are wire-visible behind this arc's outcome wall.
   **Step:** the outcome constructors and their payloads stay byte-identical; only the error side
   is re-typed.
4. **The payloads differ per variant.** **Step:** carry each variant's existing fields verbatim; a
   unified payload would lose information and change messages.
5. **`no_ceiling_raise_in_rete` stays.** It guards construction. **Step:** if its `CEILING_VARIANTS`
   strings stop matching after the re-typing, update them so the lint still fires — and **run
   `binary_id(wat::lint)`**, which has caught a red for five consecutive riders.
6. **Messages must not change.** **Step:** if any rendered diagnostic differs, that is a finding to
   report, not a silent edit.

## STOP triggers

- **STOP-1** — if the re-typing forces a change to a wat-facing outcome shape, STOP and report. Trap
  3 says it must not.
- **STOP-2** — if any currently-green test goes red, STOP and report which.
- **STOP-3** — if a fifth ceiling-shaped variant turns up that is NOT in `CEILING_VARIANTS`, STOP and
  name it. I measured four; a fifth would be a live gap and a different strike.

## Shape to copy

`docs/arc/2026/06/278-rules-engine/strike-silent-zero/` (A2b) — the same climb-to-the-type cure on
the same class of hand-discipline.

## The one thing worth more than the fix

**Tell me where this brief was thin.** Nineteen riders before you each returned a prescription of
mine that did not survive contact. The last found that my stone's stated premise for a cut was
false, and that my sketch would have regressed twice. If a step here is wrong, unnecessary, or
impossible, say it plainly.
