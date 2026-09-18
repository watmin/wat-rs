# BRIEF — read the counter D2 has never inspected, and prove the bypass sites fire

Two drives declared D2 latent. Both were end-to-end, and the row itself records that `seen_insert`
dedups the observable they needed. Build the probe the row asked for.

## Read in order

1. `docs/arc/2026/06/278-rules-engine/VIGILIA-2026-08-30-WORK-LIST.md`, the **D2** block — the trace,
   the two drives, and the closing sentence naming the probe that was never built.
2. `src/rete/kernel/fire/mod.rs:795-818` — the **only** maintainer: reads `already` at `:799`,
   appends at `:802`, writes `indexed_n` back at `:815`.
3. `src/rete/kernel/fire/pass/hash_join.rs:185` and `:298` — the two appends that **do not** touch
   the counter. `:298`'s own comment says *"Step 2: add Δright (dr) to `right_idx[J]` FIRST."*
4. `src/rete/kernel/fire/pass/mod.rs:81,128` — how `right_idx_n` reaches the passes as `indexed_n`.
5. `src/rete/kernel/tests/where_tree_branch_differential.rs` — the shape for a Rust-level probe that
   drives a real fire and inspects engine state. **Copy its harness; do not invent a second idiom.**

## Driven by the orchestrator at HEAD `974e0d859`

Three append sites, one maintainer (table in the DESIGN). The row's stated evidence — *"not even a
parameter"* — is **false today**: `partire` threaded `right_idx_n` into both passes and they still do
not bump it.

**Pre-values:** floor `5418 tests run: 5418 passed, 21 skipped` · `wat::lint` 265 · clippy rc=0.

## The change

A Rust-level probe that fires the `filter → HashJoin(a) → HashJoin(b)` shape the row already decoded
from a real `Export`, and after **each round** asserts, for every join id `J`:

```
indexed_n[J] == right_idx[J].len()
```

★ **And proves `hash_join.rs:185` and `:298` executed.** A census counter, a marker, whatever the
harness allows — but the verdict does not count until both bypass sites are shown to have run.

## STOP triggers

1. **⛔ If the invariant BREAKS, stop and report immediately.** D2 is live, the bounded negative was
   wrong, and that outranks this strike. Capture the join id, the counter, the length, and the round.
2. **If you cannot prove `:185`/`:298` executed**, stop and report. A green invariant over unreached
   code is the vacuous pass this whole arc has been hunting — do not report it as a closure.
3. **If reaching those sites needs an engine edit**, stop. C10 forbids a hot-path change for an
   instrument's benefit.
4. **If the probe needs `pub(crate)` widened**, that is acceptable — say so explicitly. Logic changes
   are not.

## Mutation proofs — run all three, report all three

1. ★ **Break the maintainer** — make `:815` write `right_elements.len() - 1` → the invariant REDs,
   naming `J`, counter, length, round. Proves the assert reads the real pair.
2. ★ **Skip one bypass site's append** (`:298`) → **the reachability half REDs**, proving it was
   genuinely executing and the non-vacuity claim is not a comment.
3. **Run the probe on a shape with no second HashJoin** → it must FAIL as inapplicable, not pass
   green. A probe that quietly measures nothing is the defect it exists to disprove.

Restore by **hash** — `git checkout <sha> -- <path>` STAGES.

## What to report

- The invariant's verdict per round, and **the evidence both bypass sites executed**.
- All three mutation results.
- Whether D2 is LIVE or a bounded negative **with an instrument** — and say which plainly.
- Scoped nextest `Summary` lines including `binary_id(wat::lint)`.
- **Anywhere this brief was thin or wrong. Be blunt.** Thirteen consecutive strikes had their ★ be a
  false claim in a file the brief said to trust — **eleven were the orchestrator's own artifacts**,
  the most recent an artifact that wrote the corrected number it forbade, six times. Assume there is
  a fourteenth.

Do not commit.
