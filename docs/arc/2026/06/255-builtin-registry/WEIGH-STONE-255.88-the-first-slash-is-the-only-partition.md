# WEIGH — STONE 255.88: the first slash is the only partition — ACCEPTED

**Executor: grok via pulsare.** Probe `f789723de` (red at birth), cure `ab9bc3f8b`, the withdrawn padding `d916e49c4` and
its reversal with re-captured spans `b56f70ba6`, SCORE `30cd8b69c`. Weighed by the orchestrator on 2026-10-03.

## Re-run by the orchestrator at `30cd8b69c`

| row | result |
|---|---|
| release floor | `.floor/2026-10-03T13-09-11Z`: **6412 passed / 24 skipped**, exit 0; doc-link step 0 |
| clippy | exit 0 |
| ignore ledger | **18** (the doc-link judge is a bin, not an `#[ignore]` test) |
| the re-captured goldens | the five `.edn` diffs, with `:line` values masked, are identical line for line: only line numbers changed |
| the padding | `wat/core.wat`'s bare `;;` line is gone |

## What landed

- **The builder's rule holds:** the first `/` partitions namespace from name; every later `/` is a name character
  (`wat.core//` is `{wat.core, /}`). The probe (the builder's three examples plus `wat.core//` and hygiene) prints
  `0 7 42 1 99`. A multi-slash declared name is callable: `core.wat`'s `defn` name block had split first-and-last segment
  (`u/a/b` registered as `:u::b`); it now uses `canonical-identity`.
- **A binder is `$bound` by position** (`Identifier::into_bound`: `let`, `fn` params, two-element `match`, destructuring,
  variant field patterns); a variant head stays a reference. A symbol name containing `/` is never a member join.
- Stale "last slash" docs corrected. The doc-link judge is `src/bin/doc_link_ledger.rs`, proven red on a planted link.
- **`flat` measured, kept:** 504,923 `as_str` calls on one probe run, ~4.3 MB a reconstruction would allocate there. Its
  removal stays the builder's later call.

## On the record

- `d916e49c4` padded `wat/core.wat` with a bare comment line so goldens kept their line numbers; withdrawn at the weigh
  (memory: the cheap-green family).
- **A fragility, recorded, not cured:** five goldens pin line numbers inside `wat/core.wat`, so any stdlib edit above them
  churns unrelated tests (255.87 +3, 255.88 −1). The SCORE names what each span is there to prove, for a later stone to
  assert the span's file and form instead.

## Verdict

Accepted and pushed. Next: 5c-iii, the corpus conversion.
