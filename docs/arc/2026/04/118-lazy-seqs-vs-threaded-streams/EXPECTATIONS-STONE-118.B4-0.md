# EXPECTATIONS — STONE 118.B4-0 · `nth` native, wat clause becomes the oracle

Written before the strike. The scorecard cannot move.

| # | what | command | expected |
|---|---|---|---|
| 1 | ★ **a macro body can call `(nth …)`** | the new macro-body probe | green — it is impossible today |
| 2 | ★ **differential `nth` ≡ `nth-spec`** | the new differential | agree on Vector/PV/List/Stream, index 0/mid/last/past-end |
| 3 | non-vacuity of the differential | perturb one side, re-run, revert | perturbed → RED; reverted → byte-identical |
| 4 | ★ Stream `nth` realizes exactly **i+1** | force-count test | `i+1`, not `n` |
| 5 | `indexable()` membership unchanged | read `seq_container.rs` | Stream still `true` — B4-iii flips it, not this stone |
| 6 | existing `nth` callers untouched | `git status --short` | `bracket.wat`, `fix.wat`, `service.wat` NOT modified |
| 7 | floor | `scripts/floor.sh` | **≥4756 run, 0 FAIL, 19 skipped** |
| 8 | clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
| 9 | ignores unchanged | the ignore census | 13 |
| 10 | blast radius held | `git status --short` | 4 `src/` files + `wat/core.wat` + new tests only |

**Rows 1 and 2 are load-bearing together.** Row 1 alone proves reachability, not correctness — a
broken `nth` is equally callable from a macro body. Row 2 alone proves correctness of something that
might still be unreachable where it is needed. Row 4 is what stops a "correct" native that drains.

## Independent prediction

**50–75 minutes.** Four `src/` files with a mirrored checker/runtime pair, plus three test shapes.
The capability method and the two classifiers are mechanical; the Stream walk and the checker's
index-typing are the real work.

## Trap-doors named in advance

- **Reusing `indexable()`** is the obvious shortcut and it is a time bomb: B4-iii flips that bit and
  would close `nth` on Streams silently, three stones later. Row 5 exists to catch a rider who took
  the shortcut.
- **A native that drains** — realize the whole stream, then index — passes rows 1, 2, and 6 and fails
  only row 4. That is exactly the retention B3 deleted, so row 4 is not optional.
- **The `infer_list` arm** this adds moves arc 255's hand-written-arm count the wrong way. Known,
  accepted, must be reported — a rider that silently adds it has hidden a real cost.
- **The oracle must stay wat.** If a rider "simplifies" by making `nth-spec` delegate to the native,
  the differential becomes a tautology and proves nothing.
  `[[feedback_an_oracle_must_be_written_in_the_other_language]]`
