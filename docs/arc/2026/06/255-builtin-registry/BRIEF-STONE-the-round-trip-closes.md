# BRIEF — STONE: the round trip closes

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Any path containing `.claude/worktrees/`
is harness state — never operate on it. Do not commit, push, stash, or revert. Do not run the full
floor; the orchestrator runs it centrally.

Read `DESIGN-STONE-the-round-trip-closes.md` (sibling) first, in particular its ⛔ section — the two
gaps are **spelling, not data**, and the orchestrator's first framing ("two defects") overstated
them.

## The work in one paragraph

`probe_can_doc_types_reconstruct_the_checker_scheme` reports 430 of 432 registered rows whose
`@arg`/`@ret` doc types reconstruct their `CheckEnv` TypeScheme exactly. Two rows fail, and both
fail on how a type is SPELLED rather than on what it means. Decide which spelling is canonical —
by measurement — make the single change each answer implies, and re-run to 432/432.

## Rooms, in order

1. **`src/intrinsic/mod.rs`, `probe_can_doc_types_reconstruct_the_checker_scheme`** — read the
   comparison closure (`let mut check = |doc, want, what|`) and the ⛔ comment above `if got !=
   *want`. That comment is load-bearing: an earlier draft compared through a lossy projection and
   scored a false 386/386. **Whatever you change, that protection must survive.**
2. **`src/types.rs:1069`** — `:wat::core::nil` is registered as `TypeDef::Alias` with
   `expr: TypeExpr::Tuple(vec![])`. This is why the doc parse yields `Tuple([])`.
3. **`src/check.rs`, `:wat::rete::lower`'s scheme** — its `ret:` is
   `TypeExpr::Path(":wat::core::nil")`, the unresolved spelling.
4. **`src/check.rs`, `:wat::string::join`'s scheme** — `type_params: vec!["T"]` and, in its
   `Seqable` arg, `TypeExpr::Path("T")` — bare, where the file's own `t_var()` closure builds
   `Path(":T")`. Compare against `:wat::core::foldl`'s scheme in the same file.

## The two questions — answer by measurement, not preference

**Q1 — `nil`.** Is the canonical form the resolved `Tuple([])` or the unresolved
`Path(":wat::core::nil")`? Measure before choosing: how do OTHER schemes in `register_builtins`
spell a nil return, and does anything else in the file use `Path(":wat::core::nil")`? If `lower` is
alone, the scheme is the outlier. If many schemes use it, the **instrument** is the outlier and the
honest fix is resolving aliases on both sides before comparing — a question about TYPES, not
SPELLINGS.

**Q2 — type variables.** `Path(":T")` or `Path("T")`? Count both spellings across
`register_builtins`. The minority spelling is the one to correct.

⚠ **If Q1's answer is "fix the instrument," this stone ships ONE data change, not two.** That is a
success, not a shortfall — say plainly in your report which of the two it turned out to be.

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — if either mismatch turns out to be a genuine SEMANTIC disagreement (the doc type and
  the scheme denote different types, not the same type spelled twice), STOP and report it. That
  would mean the registry's data is insufficient for that row, which is a materially different
  finding and a different stone.
- **STOP-2** — do not weaken the probe's structural comparison to make the number go up. Comparing
  through `typeexpr_to_doc_string` or any other projection is exactly the defect that comment
  guards, and it once produced a false perfect score. If you resolve aliases, resolve them on BOTH
  sides and leave the `TypeExpr`-level equality intact.
- **STOP-3** — do not touch any other scheme's content, the 121 DEBT rows, or `CheckEnv`'s
  consult order. Phase 3b is not this stone.
- **STOP-4** — if changing `join`'s spelling turns anything red, STOP and report. `join` is called
  across the corpus and its scheme was widened to `Seqable` by Stone D; a spelling change should be
  inert, and if it is not, the reason matters more than the fix.

## Verification

```
cargo nextest run --release -E 'test(probe_can_doc_types_reconstruct_the_checker_scheme)' --no-capture
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::types)'
cargo nextest run --release -E 'binary_id(wat::collection)'
cargo clippy --release --all-targets -- -D warnings
```

The first one is the acceptance: read `round-trip EXACTLY` and `failed` off its output.

## What to report

Your answers to Q1 and Q2 with the counts that decided them; whether the fix was data or
instrument (and for Q1, which); the probe's before/after `round-trip EXACTLY` and `failed` lines
verbatim; the Summary line per scoped run; and anything that surprised you.
