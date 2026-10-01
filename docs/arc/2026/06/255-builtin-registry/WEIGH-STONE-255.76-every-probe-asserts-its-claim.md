# WEIGH — STONE 255.76: every probe asserts its claim — ACCEPTED

**Executor: a Sonnet subagent, commits `cf4d5b00e` (the stone), `79f278097` and `e266c8e1a` (corrections).** Weighed by
the orchestrator on 2026-10-01.

## Re-run by the orchestrator at `e266c8e1a`

| row | result |
|---|---|
| release floor | `.floor/2026-10-01T08-16-09Z`: **6336 passed / 24 skipped**, exit 0; 69 `every_probe_runs::` tests pass |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 |
| no bare print-only probe | every probe without an assertion carries `CLAIM (exit 0)`: `probe-054-fn-idempotency`, `probe-compound-upcast`, `probe-nested-vector-of-tuples`, `probe-s3a-select-peer`, `probe-trivial` |
| no ascription added | `git diff 561b8d46b..HEAD -- '*.wat'`: no added `ann-form` outside comments |
| string-literal asserts | four remain, each the probe's own computed `String` value (`"sqlite 2067"` ×2, `"even"`, `"odd"`) |

## What landed

39 print-only probes: **33 assert** on data (five mutation-proved, each red by its own generated test name), **5
check-claim** (exit 0 is the proof, said in the header), **1 retired** (`probe-edn2.wat`, a byte-identical copy of
`probe-edn.wat`). Every kept probe's header ends on a one-line `CLAIM`. Two measured answers are now pinned:
`probe-type-splice` (a generic fn's type parameter is not substituted into its `forms` quote; the header says the red
the day it is, is the signal to rewrite) and `root-gapA` (no divergence between `fn-forms` on a hand-written and a kwargs
`$impl` fn).

## Corrections the weigh made

- Five assertions compared strings standing in for data (a rendering in `probe-edn`, `ast-name`/`ast-kind` text in
  `probe-fnforms-shape`, `probe-s3b-extract`, `probe-type-splice`, and `root-gapA`, which the agent found itself); and
  `strikeB-fields` checked only the count of deterministic field types. Converted to AST-node equality, a data round trip,
  and the exact types.
- The first `probe-edn` fix pinned its round trip with `ann-form` (ascription); replaced by a typed consumer.

## Open: the ascription class (for the builder)

`ann-form` entered executor work twice on 2026-10-01 (255.74's D3 pass, this stone), each time against a brief or ruling
that said "no ascription". That is the convention rung failing. 46 uses remain in `.wat`, some of them a probe's own
subject (`probe-compound-upcast`, `probe-m1-erase-only`). A ratchet (the count can only fall; a new use is a floor red)
would put it on the check rung.

## Verdict

Accepted and pushed. E3 is complete: every probe runs on the floor, and exit 0 means its stated claim held.
