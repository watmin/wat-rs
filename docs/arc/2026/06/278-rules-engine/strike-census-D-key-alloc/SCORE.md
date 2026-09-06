# SCORE — `match:key-alloc` is `bindkey:alloc` now

One mechanism, one site, three callers. The name no longer claims a provenance the counter never had. No engine behaviour, no signature, no new counter. Floor GREEN.

## Scorecard

| # | result |
|---|---|
| 1 ★ rename complete | **HOLD.** `grep -rn '"match:key-alloc"' src/ tests/` → no hits. The leftover `match:key-alloc` in `matcher.rs:680` is a comment *without* double quotes, naming the rejected prefix. |
| 2 ★ the counter is still live | **HOLD.** `compiled_cond_failure_path_allocates_no_binding_keys_at_50_100` under the new name. Quoted below. Consumers that assert ZERO cannot red a deleted bump — that is said, not manufactured. |
| 3 ★ the false comment is true | **HOLD.** `eval_insert.rs:154-156` now says the class-name String is NOT counted by `bindkey:alloc`, AND this file's own RHS resolution IS, through `resolve_operand` (`resolve_rhs_value`). |
| 4 ★ no behaviour change | **HOLD.** Two `&'static str` literals and comments. `resolve_operand`'s signature is byte-identical. No new counter. |
| 5 ★ the rejection is recorded | **HOLD.** Bind-arm comment at `matcher.rs:680-685`: per-caller attribution (a `&'static str` parameter on `resolve_operand`) considered and rejected — no consumer wants it, C10 (`accum_cost.rs`) holds discriminating for an instrument's benefit is an engine edit. |
| 6 harness condition | **HOLD.** `alpha_discrimination.rs:341-343` states the site attribution holds HERE because this harness arms the census around direct matcher calls only; a whole-fire read also includes RHS insert and step-payload. |
| 7 `fanout_cost.rs` guards | **HOLD.** Label `bindkey:alloc (RHS + alpha, both compiled — expect 0)` and `prod:derivations == 40_000` survive verbatim. Quoted below. |
| 8 floor | **HOLD.** Final `5465 passed, 21 skipped` — same count as census B. No new tests. |
| 9 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |

★ load-bearing. **Row 2 is the only liveness this counter has; row 5 is what stops the rejected option being re-derived as a good idea.**

## Row 2 — liveness (honest substitute, not a mutation proof)

Consumers assert ZERO (`fanout_cost.rs` whole-fire compiled; `alpha_discrimination.rs` compiled path). Deleting a bump keeps them green. The DESIGN forbade fabricating a red they cannot support. The interpreter reading under the new name is the gate that exists:

```
  ROW 2 — failure-path binding-key allocation, [50 100] cascade
  compiled calls:    10000 (9800 failed, 98.0% failure rate)
  compiled path    bindkey:alloc = 0
  interpreter      bindkey:alloc = 30000   (over 10000 calls, the SAME corpus)
```

`interp_key_allocs = 30000 > 0`. Compiled stays 0. Same corpus, same test, new name.

Fanout (zero, as designed — label honesty kept):

```
  FANOUT RHS ALLOCATION CENSUS — keys=100 x fanout=20, 40,000 derived Pairs

  bindkey:alloc (RHS + alpha, both compiled — expect 0)            0
  per derived fact                                             0.00
  match:calls (interpreter entries — expect 0)                    0
  prod:derivations (non-vacuity guard — expect 40,000)        40000
```

No way found to red a consumer by deleting a bump. Reported as the DESIGN predicted.

## First floor (captured, not re-run)

`.floor/2026-09-06T10-51-01Z/`: `Summary [ 461.471s] 5465 tests run: 5464 passed (3 slow), 1 failed, 21 skipped`.

Arm: `wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves`.

```
🔥 1 name(s) cited in a comment under src/rete resolve to NOTHING

Unresolved:

  src/rete/matcher.rs:682  `census_key`
```

The Bind-arm comment had backticked a never-minted parameter name. Cure is option 2 of that gate: reword. The rejection is now "a `&'static str` parameter on `resolve_operand`" — `resolve_operand` exists; `census_key` never did. Backticks were not deleted around a hollow name.

## Final floor

`.floor/2026-09-06T11-05-55Z/`: `Summary [ 460.907s] 5465 tests run: 5465 passed (3 slow), 21 skipped`. Count unchanged. No new tests.
