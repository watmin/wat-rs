# EXPECTATIONS — census D

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | expected |
|---|---|---|
| 1 ★ | rename complete | `grep -rn '"match:key-alloc"' src/ tests/` → **no hits** |
| 2 ★ | the counter is still live | `alpha_discrimination.rs`'s `interp_key_allocs > 0` passes under the new name — quote the printed row |
| 3 ★ | the false comment is true | `eval_insert.rs:154-155` now says its own RHS resolution DOES bump the counter |
| 4 ★ | no behaviour change | no engine value moves; no signature change; no new counter |
| 5 ★ | the rejection is recorded | per-caller attribution named as considered-and-rejected at the declaration, citing C10 |
| 6 | the harness condition is stated | `alpha_discrimination.rs`'s doc says its attribution holds because the census is armed around direct matcher calls |
| 7 | `fanout_cost.rs` keeps its guards | the `(RHS + alpha)` label and `prod:derivations == 40_000` survive verbatim |
| 8 | floor | **≥ 5465 — tell me the final number and what moved it.** New arms are a PASS; name them |
| 9 | clippy | rc=0 |

★ load-bearing. **Row 2 is the only liveness this counter has; row 5 is what stops the rejected
option being re-derived as a good idea.**

## Trap doors, named in advance

- **A rename that misses a reader.** The census-name lint turns that into a build failure rather
  than a silent `unwrap_or(0)` — do not add a second gate for it.
- **"Improving" the counter into per-caller keys.** Rejected in DESIGN, with the reason. Doing it
  anyway is the strike failing, not exceeding.
- **Weakening `fanout_cost.rs`'s non-vacuity guard** while touching its reads.
- **Claiming a mutation proof the consumers cannot support** — they assert zero, so a deleted bump
  keeps them green. Row 2 is the honest substitute; say so rather than manufacturing a red.
- **Re-running a red.** Capture whole, name the arm, surface it.

## What would make me reject the result

- `match:key-alloc` still emitted anywhere in code.
- Any engine behaviour or signature change.
- A fabricated mutation proof.
- The rejection of per-caller attribution left unrecorded.
