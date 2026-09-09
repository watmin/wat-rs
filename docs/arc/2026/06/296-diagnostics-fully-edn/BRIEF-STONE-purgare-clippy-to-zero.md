# BRIEF — STONE purgare: clippy to ZERO

## The standing rule the record got wrong

**Builder, 2026-09-08:** *"we've been at zero clippy issues for months — we do not tolerate >0."*

I called these five *"the same 5 pre-existing dead_code items"* in five consecutive commit messages
and in the seam. **That was a dismissal, not a disposition** — the same one `scripts/floor.sh`'s own
header forbids for tests (*"'pre-existing' describes your search, not the failure"*), applied to
clippy without my noticing I was doing it.

## The five, and what they are

```
src/check.rs     fn pattern_coverage is never used
src/check.rs     variant Coverage::Wildcard is never constructed
src/runtime.rs   fn try_match_pattern_ast is never used
src/runtime.rs   fn substitute_many is never used
src/match_arm.rs field ident_span is never read
```

Measured: all five lost their call sites at `480f38d05` (arc 296 stone L, 2026-09-07 — whose own
commit message claims `clippy 0`). They are **SUPERSEDED**, not accidentally orphaned:

- `pattern_coverage` → replaced by `cover_variant_arm`, which is live at `src/check.rs:6407`.
- `try_match_pattern_ast` / `substitute_many` → arc 068 β-reduction step rules, replaced by the
  current evaluator.

The floor passes **5291/5291** with none of them called.

## The work

Delete them, then follow the compiler. **Deletion cascades**: removing `pattern_coverage` may orphan
`Coverage` itself, its helpers, or their imports. Build, read the new `dead_code` warnings, delete
those, repeat until `cargo clippy --release --all-targets -- -D warnings` **exits 0**.

⛔ **DELETE ONLY.** No behaviour changes, no signature changes, no "while I'm here". If removing
something requires changing what any live code does, **STOP and report** — that means it was not
dead and the diagnosis was wrong.

## Report the cascade before finishing

Write `TABLE-STONE-purgare-the-cascade.md`: each item deleted, which round it surfaced in, and what
orphaned it. A five-item fix that cascades to twenty is a different stone and the orchestrator wants
to see the shape, not just the zero.

## Acceptance

```
cargo clippy --release --all-targets -- -D warnings       EXIT 0
cargo nextest run --release -E 'test(a2_a_variant)'       16 passed, 0 skipped
```

Plus the five stone filters unmoved: `p1_annotation` 10 · `p1b_a_parametric` 4 · `p2prereq` 4 ·
`p3_one_question` 5 · `a1_one_rule` 4.

## STOP triggers — each is a REJECTION

**STOP-1.** If deleting anything requires a behavioural change — STOP. It was not dead.

**STOP-2.** If the cascade exceeds ~20 items — STOP and report the table. That is a campaign.

**STOP-3.** If a `#[allow(dead_code)]` would make clippy green — **STOP.** Suppressing the wall is
the opposite of the work. Every exemption in this repo is audited against present truth
(`excusare`), and "it is dead but I muted the warning" earns no standing.

**STOP-4.** If an item looks dead to clippy but is reachable through a macro, `#[cfg]`, a test-only
path, or reflection — STOP and report which. Clippy cannot see every caller, and a wrong deletion
here is a silent capability loss.

## Tier

You edit and report. Run clippy and the targeted `-E` filters — clippy IS your gate for this stone,
unusually, because it is the subject. **You do NOT run `scripts/floor.sh`**; the orchestrator runs it
centrally afterwards.
