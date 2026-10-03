# WEIGH — STONE 255.87: cutover 5c-ii — the stdlib speaks faithful Clojure — ACCEPTED

**Executor: grok via pulsare, through six amendments.** Conversion `50f4c0a52`; group A `977efcbfd`…`731e24740`; group
B `281cdd483`…`0f10d248b`; the cost `369162eb2`; names `bdb9953d7`; own reds and the doc step `b2156b785`; SCORE
`d131e2833`. Weighed by the orchestrator on 2026-10-03.

## Re-run by the orchestrator at `d131e2833`

| row | result |
|---|---|
| release floor | `.floor/2026-10-03T11-40-59Z`: **6410 passed / 25 skipped**, exit 0, **377.6 s** (the pre-conversion floors ran 394–407 s) |
| doc-link step | `floor.sh`'s new step: `doc-link exit=0`, log `.floor/2026-10-03T11-40-59Z/doc-link.log` |
| clippy | exit 0 |
| delta (grok) | `NEW 0`, `RECOVERY 0` on the committed 178-path sample; census `no STOP-8` |

## What landed

- **The 65 stdlib files speak faithful Clojure** (11,131 keyword heads → 0), by the recorded converter run from a pristine
  binary, as its own commit.
- **Group A** (substrate): a symbol declaration name registers as its canonical identity; `wat.core/def` is lifted like
  `:wat::core::def`; a symbol type in a fact-bind is not an accumulator; closure extraction and kwargs minting read the
  symbol; a Rust method's member join is the key the registry holds (the sqlite/journal `disconnected` was that death,
  hidden behind the client's scrub).
- **Group B** (readers and goldens): grep, the nested-program census, the stdio gate, `structtype`, the emitted-binder
  census read by identity; the faithful-surface gate counts member joins; span goldens re-captured after an as-data audit.
- **The cost:** the conversion first cost **1.47×** startup. Cures (`is_subtype_parent` a set; a symbol's spelling stored
  once at registration) brought it to a six-run residual of **1.107×** (shard 2: 9.09 vs 8.21 s) and **1.090×**
  (`keyed_gather`), with the converted tree already faster than the morning's unconverted baseline. The residual is
  accepted (P1); the larger win is a separate design (one frozen stdlib shared across worlds). No limit was raised.
- **A name is not a substring:** companion names (`::kwargs-check` and kin) are minted from `canonical-identity` plus the
  suffix, never by slicing text, across six stdlib files.
- **The doc-link check left the 30 s nextest slot (D1):** a `floor.sh` step on the floor's own target, the same
  frozen-by-name ledger, proven red on a planted broken link.

## Owed (carried into 255.88)

- **The ignore ledger rose 18 → 19:** the doc-link judge is an `#[ignore]` test that `floor.sh` runs explicitly
  (`tests/lint/no_new_broken_doc_link.rs:277`). The builder's ruling is one ignore at the end; the judge becomes a
  non-test entry (a small binary or the script itself), not an ignored test.
- P2 was withdrawn: under the builder's slash rule a slashed body symbol can name a local, so references keep scopes.

## Verdict

Accepted and pushed. 5c-ii is complete. Next: 255.88 (the first slash is the only partition), then 5c-iii.
