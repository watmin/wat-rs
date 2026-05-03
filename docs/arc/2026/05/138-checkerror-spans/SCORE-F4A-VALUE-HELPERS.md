# Arc 138 F4a — SCORE

**Written:** 2026-05-03 AFTER sonnet's report.
**Agent ID:** `adc1007ff7d07e989`
**Runtime:** ~14 min (815 s) — within 10-20 min prediction.

## Verification

| Claim | Disk-verified |
|---|---|
| Files modified | 3 (src/spawn.rs, src/string_ops.rs, src/io.rs) ✓ |
| diff stat 85+/75- | ✓ |
| All 11 helpers updated | spawn 4 + string_ops 1 + 4 bonus + io 5 = 14 helpers ✓ |
| Span::unknown() pre/post | spawn 6→0, string_ops 7→2, io 25→19. Total 38→21 (45% drop) ✓ |
| 6/6 arc138 canaries | PASS ✓ |
| Workspace tests pass | empty FAILED ✓ |

## Hard scorecard: 6/6 PASS. Soft: 3/3 PASS+.

## Substrate observation — string_ops over-delivery

BRIEF listed only `two_strings`. Sonnet found 4 additional in-file sites where spans were derivable from `args` and fixed them all (one_string, eval_string_join arity, eval_string_split sep-empty, eval_regex_matches compile error). All same-file, no cross-file broadening. Honest expansion of scope.

## Substrate observation — F4a closes most of the Value-shaped gap

Three sub-cracks in F4 charter:
- F4a (this slice): Value-shaped helpers in spawn/string_ops/io — CLOSED
- F4b: FromWat trait in marshal.rs — pending
- F4c: ThreadOwnedCell::with_mut — pending

io.rs leftover 19 split: arity helper (out of F4a scope), snapshot_writer helper (out of scope), loader/path OS errors (genuine Pattern E), test-code Span::unknown() (correct synthetic context).

## Calibration

Predicted 10-20 min; actual 14 min. In-band.

## Ship decision

**SHIP.** Next: F4b (FromWat trait expansion).
