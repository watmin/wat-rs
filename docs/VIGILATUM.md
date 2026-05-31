# VIGILATUM — the ward-provenance marker

> *vigilatum* — Latin, past participle of *vigilo*: "watched," "guarded," "kept vigil over." Where [`vigilia`](../../datamancy.dev/vigilia/SKILL.md) is *the watch*, `vigilatum` is *the watch having passed here.* Named by intueri cast, 2026-05-30.

## What it is

A single module-doc line that records a namespaced home's **ward provenance**: when the home last passed the vigilia REMARKABLE bar (L1 + L2 = 0), and what cast it. The anchor commit is not written in the line — git already holds it (see *Drift* below).

```rust
//! vigilatum: 2026-05-31T08:24:29Z — vigilia 8-spell L1+L2=0
```

"Watched here. At this instant. By vigilia. Zero divergence." The line is its own definition — no glossary, no lookup.

## Why it exists

The substrate matures by selective lift-and-ward (`feedback_selective_lift_and_ward`): flat `src/*.rs` is functional-but-untrusted; a thing is lifted into a namespace home when it is brought to the REMARKABLE bar. `vigilatum` answers the question that was previously only answerable by guessing at file structure or reading git archaeology: **is this home actually warded, and has it drifted since?**

### Drift — git is the anchor, not the line

The stamp ships **in the same commit as the warded code** (one atomic commit — see *Why no commit hash* below). So the ward commit IS the drift baseline, and git recovers it directly:

```
anchor=$(git log -1 --format=%H -G'vigilatum:' -- src/<home>/mod.rs)
git diff "$anchor"..HEAD -- src/<home>/
```

An **empty** diff = warded and unchanged. **Any** hunk = the home was touched since it was warded = a re-ward is owed. The marker is past-tense and bounded to a moment (intueri chose `vigilatum` over ongoing-state candidates like `custodia` precisely for this): it claims the watch *passed*, never that the code is *still* guaranteed — the diff is the instrument that discovers whether the moment has aged.

### Why no commit hash

The marker once embedded `@ <commit>` — the hash of its own ward commit. That is a chicken-and-egg: the hash cannot exist until the commit exists, but the line lives *inside* the file that commit contains. It forced a two-commit dance (ward the code, then a second commit to write the hash) and a read-back step — and the read-back, shortcut from expectation, fabricated non-existent hashes four times in one session. A mechanism that manufactures false claims fails the substrate's first rule. The fix eliminates the class, not the symptom: **drop the hash.** Git already holds the provenance; the home wards in one honest commit; there is no hash to fabricate.

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
//! vigilatum: <ISO8601-UTC-seconds> — vigilia <N>-spell L1+L2=0
```

- `<ISO8601-UTC-seconds>` — the instant the watch passed, second-resolution UTC (e.g. `2026-05-31T06:18:14Z`). For a NEW ward: compute it (`date -u +%Y-%m-%dT%H:%M:%SZ`) immediately before writing the stamp, then commit — the stamp instant is the convergence instant, a few seconds ahead of the commit instant; both honest. For a RETROFIT or recovered timestamp: lift it from the home's ward commit, not from memory — `TZ=UTC git show -s --date=format-local:'%Y-%m-%dT%H:%M:%SZ' --format=%cd <ward-commit>`. Git holds the truth; the marker surfaces it.
- `vigilia <N>-spell L1+L2=0` — the verdict + how many spells stood the watch

The marker carries no commit hash — git is the anchor (see *Drift*). The stamp ships in the ward commit itself.

## What it is NOT

- NOT a central ledger. A `docs/warded.md` list would be a MIRROR of file-truth — the same duplication class CheckEnv's borrow redesign annihilated. The marker lives in the file; the file is the single source of truth.
- NOT a quality badge applied by judgment. Only a cast earns it.
- NOT permanent. It ages the instant the home is touched; the diff reveals the aging.

## Cross-references

- `datamancy.dev/vigilia/SKILL.md` — the watch this marker records
- `feedback_selective_lift_and_ward` — how homes grow (the marker is the home's ward record)
- `feedback_namespaced_home_vigilia_gate` — the L1+L2=0 bar the marker attests
- `scratch/FAILURE-ENGINEERING.md` — why an unbacked attestation is forbidden
