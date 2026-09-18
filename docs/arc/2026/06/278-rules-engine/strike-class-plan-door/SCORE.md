# SCORE — A8: the class-plan door

Push-or-demote is one act. `has_mixed` is derived. The bypass is a compile error. Floor GREEN.

LATENT: I did not construct a live demote-without-gating. Today's two arms were already paired. The defect was that the pairing was a convention.

## Scorecard

| # | result |
|---|---|
| 1 ★ one door | **HOLD.** `ClassPlan::observe` is the only mutating verb. Packed → push ids. Unpacked → `uniform = false`. One call. |
| 2 ★ `has_mixed` DERIVED | **HOLD.** No `any_mixed` field. `has_mixed()` is `self.map.values().any(\|e\| !e.uniform)`. There is no field to set. |
| 3 ★ compile error | **HOLD.** `let _ = plan.map.len();` in `alpha_seed` → `error[E0616]: field \`map\` of struct \`ClassPlan\` is private` at `alpha.rs:177`. Reverted after capture. |
| 4 ★ batch fast path unmoved | **HOLD.** `accum_alpha_memory_shape`: `last.alpha_elements == 80_200` (printed 80200). `seed_batches_uniform_classes_and_defers_mixed_ones` green both ways. STOP-1 did not fire. |
| 5 packed-arm-first | **HOLD.** `observe` branches `if packed` before `get_mut`. |
| 6 floor | **HOLD.** `Summary [ 453.540s] 5438 tests run: 5438 passed (2 slow), 21 skipped`. `.floor/2026-09-05T22-12-43Z/`. |
| 7 clippy | **HOLD.** `cargo clippy --all-targets --release -- -D warnings` rc=0. |
| 8 blast | **HOLD.** `src/rete/kernel/fire/pass/alpha.rs` only. |

★ load-bearing. Row 6 is the deliverable.

## First floor was RED — captured, not re-run

`.floor/2026-09-05T22-00-42Z/` — `Summary [ 453.755s] 5438 tests run: 5437 passed (2 slow), 1 failed, 21 skipped`.

Arm: `wat::lint rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves` at `alpha.rs:91` citing `` `any_mixed` ``, which the strike deleted. Fix was `rune:lint(cited-name-absent) any_mixed` (absence is the point). The red floor was not re-run. Final floor is the row-6 deliverable.

## Doors

| door | why it is safe |
|---|---|
| `ClassPlan::observe` | push-or-demote, one act; `#[inline]`; packed-arm-first; takes `&mut self`, not `wm` |
| `ClassPlan::seeded` | constructor; reservation size unchanged (`temperare` L2-e is cut) |
| `ClassPlan::get` / `is_mixed` / `has_mixed` | shared reads; cannot demote |

No `#[cfg(test)]` hatch. No `&mut` to the map escapes. Nested module so `alpha_seed` in the parent cannot reach `map`.

`observe` does not take `wm` (STOP-2). `has_mixed` walks K leaf-class entries once after the fact loop, not N facts; it did not move a cost gate (STOP-3 not taken).

## Still open (named, cut)

`Vec::with_capacity(input_facts.len())` per class (`temperare` L2-e). D2p, F2, A3, A4.
