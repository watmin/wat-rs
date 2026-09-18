# EXPECTATIONS — A3: the slot zip

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | the pair has ONE form | `from_parts` takes the zip; two independent arrays cannot be handed in |
| 2 ★ | the wire hand-check is gone as a hand-check | it becomes a parse into pairs; the malformed error still fires on a mismatched import |
| 3 ★ | compile error | constructing with mismatched sequences does not compile. **Quote it** |
| 4 ★ | the silent guard is CONVERTED, not deleted | `materialize_into`'s `i >= slot_keys.len()` becomes a `debug_assert!` matching its sibling arm |
| 5 | a malformed import still fails | drive an import with `slot_keys.len() != output_slots.len()` and show the malformed error |
| 6 | wire ABI unchanged | `pack_compiled_cond` still writes two sequences at 3 and 4; round-trip green |
| 7 | floor | **0 failed** |
| 8 | clippy | rc=0 |
| 9 | cost | `accum_cost`'s `materialize_into` bench unmoved, or the delta surfaced |

★ load-bearing. **Row 7 is the deliverable; row 5 is the one that proves the cure kept the behaviour.**

## Trap doors, named in advance

- **Changing the wire ABI to make the in-memory cure easier.** STOP-1. The wire is a compatibility
  surface; the zip is not.
- **Deleting the guard.** It records what it prevented. STOP-3.
- **Locality.** Two `Arc<[T]>` scanned in lockstep versus one `Arc<[(Value, usize)]>` is a real
  layout change on a benched hot path. If a number moves, surface it (STOP-2) — do not adjust a gate.
- **This is LATENT.** The safe writer is safe by construction and the wire writer checks by hand, so
  no fire reaches the mismatch today. Do not manufacture a "before" red; the proof is the compiler.
- **Re-run the floor at FINAL state.** Four gates have fired unexpectedly across this session.

## What would make me reject the result

- Row 3 answered with anything but a compiler error.
- The wire ABI changed without surfacing it first.
- The guard deleted rather than converted.
- A malformed import that no longer fails (row 5 silently lost).
- A red floor of any size.
