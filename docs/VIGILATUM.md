# VIGILATUM — the ward-provenance marker

> *vigilatum* — Latin, past participle of *vigilo*: "watched," "guarded," "kept vigil over." Where [`vigilia`](../../datamancy.dev/vigilia/SKILL.md) is *the watch*, `vigilatum` is *the watch having passed here.* Named by intueri cast, 2026-05-30.

## What it is

A single module-doc line that records a namespaced home's **ward provenance**: when, and at which git commit, the home last passed the vigilia REMARKABLE bar (L1 + L2 = 0), and what cast it.

```rust
//! vigilatum: 2026-05-30 @ 22c89e04 — vigilia 8-spell L1+L2=0
```

"Watched here. At this commit. By vigilia. Zero divergence." The line is its own definition — no glossary, no lookup.

## Why it exists

The substrate matures by selective lift-and-ward (`feedback_selective_lift_and_ward`): flat `src/*.rs` is functional-but-untrusted; a thing is lifted into a namespace home when it is brought to the REMARKABLE bar. `vigilatum` answers the question that was previously only answerable by guessing at file structure or reading git archaeology: **is this home actually warded, and has it drifted since?**

The commit anchor is load-bearing. With `@ <commit>` recorded:

```
git diff <anchor>..HEAD -- src/<home>/
```

answers drift directly. The ONLY expected hunk is the `vigilatum` line itself (added one commit after the convergence anchor). **Any other hunk = the home was touched since it was warded = a re-ward is owed.** The marker is past-tense and bounded to a moment (intueri chose `vigilatum` over ongoing-state candidates like `custodia` precisely for this): it claims the watch *passed* at the anchor, never that the code is *still* guaranteed — the diff is the instrument that discovers whether the moment has aged.

## The iron rule — EARNED, never asserted

**A `vigilatum` line is written ONLY when a live vigilia cast has just confirmed L1 + L2 = 0 on that home.** It is an attestation. An attestation you cannot back with a cast is the exact false-claim failure mode the whole substrate refuses (`scratch/FAILURE-ENGINEERING.md`). You never backfill `vigilatum` from memory or from "it was probably fine" — git evidence (2026-05-30) showed several homes had drifted since their last convergence without anyone noticing. Re-cast, confirm, then inscribe.

**Re-ward on every touch.** When a home is modified, its `vigilatum` is stale by definition — the next work on that home re-casts vigilia and re-inscribes the line with the new anchor. "Run them into submission on every go."

## Placement — the marker follows the ward, not the filename

The line lives on the **root of the warded unit**:

- **mod.rs-rooted home** (`src/<noun>/mod.rs` + residents) → on `mod.rs`. The module root claims the home's ward state.
- **Lone resident under a flat untrusted root** (e.g. `src/check.rs` flat + `src/check/env.rs` warded) → on the resident file (`env.rs`). The flat root stays untrusted; only the warded resident carries the stamp.

The marker is the FIRST `//!` line of the file's module doc, followed by a blank `//!` line, then the existing doc.

## Inscription form

```
//! vigilatum: <YYYY-MM-DD> @ <short-commit> — vigilia <N>-spell L1+L2=0
```

- `<YYYY-MM-DD>` — the cast date (human-legible)
- `<short-commit>` — the convergence anchor (the commit at which the home passed; the drift baseline)
- `vigilia <N>-spell L1+L2=0` — the verdict + how many spells stood the watch

## What it is NOT

- NOT a central ledger. A `docs/warded.md` list would be a MIRROR of file-truth — the same duplication class CheckEnv's borrow redesign annihilated. The marker lives in the file; the file is the single source of truth.
- NOT a quality badge applied by judgment. Only a cast earns it.
- NOT permanent. It ages the instant the home is touched; the diff reveals the aging.

## Cross-references

- `datamancy.dev/vigilia/SKILL.md` — the watch this marker records
- `feedback_selective_lift_and_ward` — how homes grow (the marker is the home's ward record)
- `feedback_namespaced_home_vigilia_gate` — the L1+L2=0 bar the marker attests
- `scratch/FAILURE-ENGINEERING.md` — why an unbacked attestation is forbidden
