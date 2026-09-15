# SCORE 5 — replay batch 2: grok-rete #61 → #125

Branch: `replay/grok-rete`. **Committed, not pushed.** Main untouched.
Parent brief: `BRIEF-5-replay-batch-2.md`. Start census `.census/2026-09-15T04-07-25Z.txt` files=2054.
HEAD: `00cc59ff4` (two FIX after #125 `848a2bf6e`).

```
#93  scripts/floor.sh   .floor/2026-09-15T05-28-34Z
     Summary [225.422s] 5530 tests run: 5530 passed, 22 skipped   exit=0
     clippy 0

#125 first floor (captured, not re-run)  .floor/2026-09-15T06-54-24Z
     doctest RED: src/rete/vocabulary.rs - rete::vocabulary::Ret (line 227)
     FIX 1c6d6af3e  ```text``` fence

#125 second floor (captured, not re-run)  .floor/2026-09-15T06-55-06Z
     Summary [228.234s] 5540 tests run: 5538 passed, 2 failed, 22 skipped   exit=100
     FAIL every_row_is_admitted
     FAIL registry_membership_gap_a_is_named_and_frozen
     FIX 00cc59ff4  RETE_MODULES + GAP_A

#125 + FIX  scripts/floor.sh   .floor/2026-09-15T07-01-22Z
     Summary [227.766s] 5540 tests run: 5540 passed, 22 skipped   exit=0
clippy cargo clippy --release --all-targets -- -D warnings           CLIPPY_RC=0
```

## EXPECTATIONS

| # | result |
|---|---|
| E1 | **PASS.** `git log --oneline d4a2b1fe7^..HEAD \| grep -c 'REPLAY(grok-rete #'` = **65**. #61=`d4a2b1fe7` … #125=`848a2bf6e`. Each has `(cherry picked from commit <C>)`. |
| E2 | **PASS.** The 30 docs-only steps (#69 #70 #73 #75 #77 #79 #81 #84–#87 #89 #92 #97 #102 #104 #107 #109 #111 #113–#121 #123 #125) are cherry-pick -x of docs/`.md` only. |
| E3 | **PASS.** Every produced `.wat` `--check` rc 0. convert.sh UNREADABLE era docs expected, not STOP. No UNREGISTERABLE `wat/`. |
| E4 | **PASS.** Named tests by name on the large shared steps (#90 #91 #93 #94–#96 #98–#101 #105 #108 #110 #122) all green at the step. Sample ≥12. |
| E5 | **PASS.** Shared-step named tests green. Ownership: main syntax + non-rete; grok rete re-expressed. Goldens kept HEAD. |
| E6 | **PASS** after two captured reds. #93 green 5530/5530 clippy 0. #125 green 5540/5540 clippy 0 at `00cc59ff4`. |
| E7 | **PASS.** Census+gate on all 33 qualifying steps; lint-subset 142 on every `.rs` step. Nested-program gate isolated ~29s. No STOP-8, no STOP-10. |
| E8 | **PASS.** No step touched `wat-scripts/fixes/`, `wat/`, or a file main deleted. |
| E9 | **PASS.** Corpus `.wat` from convert.sh + merge-file. wat-in-rs-strings rewritten (logged). LATENT #95 keyword homes logged. |
| E10 | **PASS.** `census.sh --diff .census/2026-09-15T04-07-25Z.txt .census/latest` → `census-diff: no STOP-8`. Latest `.census/2026-09-15T06-53-02Z.txt` files=2077. |

## Re-expression that mattered

- **#101 ONE rule.** convert.sh / rewrite mapped grok's unit-variant keyword `:probe::E::A` to constructor `(:probe::E.A {})`. The ONE-rule path only lowers `WatAST::Keyword`. defenum unit is `:A :B` (no `[]` — that makes tagged-empty and types the keyword as `[:-> …]`). `keyword_constant_segment` uses `decompose_variant`.
- **#108 Ret.** grok's `classify_fallback_outcome` still lived in `runtime.rs`; HEAD home is `src/holon/outcome.rs`. Goldens kept HEAD. HEAD-only `variant-name` wrapped `Ret::Is(String)` and named in the scheme-less freeze list.
- **#110 `#holon` fold.** `eval_quote` already `pub(crate)` on HEAD. `to_holon_inner` via `crate::holon`.
- **#122 hash-destructure.** grok Map arm **and** HEAD KEY-FIRST Vector nested-variant arm.

## Captured reds (not re-run)

1. `.floor/2026-09-15T06-54-24Z` doctest: grok's three-site reader list was four-space indented under `///`, so rustdoc compiled it as Rust. Fence tagged ` ```text `.
2. `.floor/2026-09-15T06-55-06Z` ARM verbatim: GAP_A NEW `[:wat::rete::keyword::from-string, :wat::rete::keyword::to-string]`; `every_row_is_admitted` row `:wat::rete::keyword::to-string` not in RETE_MODULES.

## STOP

None remaining. Do not push. Main untouched.
