# EXPECTATIONS — type-check `accumulate`'s `:from` clauses

Written **before** the strike, at `944bdd959`.

| what | command | expected |
|---|---|---|
| ⭐ **a legal `accumulate` compiles and fires TODAY** | the positive fixture, **before** any source change | **it runs.** If this cannot be built, STOP-2 fires and the strike ends here |
| the gap closes | the refusal fixture | ill-typed predicate in `:from` → refused, naming the type mismatch |
| the inline twin is unchanged | the control | still `ConstraintTypeMismatch` |
| ⛔ **the over-rejection guard holds** | the positive fixture, **after** the change | still compiles and fires — including its not-knowable operand |
| the retired claim is recorded | `git diff src/rete/validate/mod.rs` | the `:296` comment **rewritten**, not deleted |
| ⛔ **MUTATION — the check is load-bearing** | revert the arm to `validate_fact_type_head_only` **in the live source**, re-run | the refusal test **REDs**. Restore ⇒ green. Mutate the shipping code, never a copy |
| ⛔ **MUTATION — the guard is load-bearing** | make the arm refuse every operand it cannot type | the **positive** fixture REDs. If it does not, the guard is not testing what it claims |
| the sibling arms are untouched | driven | `not` inner, `exists` inner, `or`, `and`, `:then` all still refuse — they did before this strike and must after |
| scope held | `git status` | no change to `reachability.rs`, `typing.rs`, `clause.rs`, or `wat-scripts/` |
| floor | `./scripts/floor.sh`, result read by you | **0 failed.** Passed ≥ 5516 + the new tests. Do not pin an exact total |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

**60–90 minutes**, and the variance is entirely the positive corpus. The source change is one line.
If `accumulate` cannot be exercised at all from the spec, the strike ends at STOP-2 with that
finding — which is a legitimate and valuable result, since it would mean the form is not merely
unused but **unusable**.

## Trap doors

- ⛔ **Shipping the check without the positive corpus.** There are zero corpus uses, so an
  over-rejecting cure REDS NOTHING. The floor stays green while the surface silently narrows. This
  is the single most likely way this strike ships broken and looks finished.
- ⛔ **Assuming `Design call 3` was a mistake because its sibling was.** The `Where` arm's identical
  justification hid a real hole; `reachability.rs`'s identical-looking exclusion was correct,
  argued, and tracked — and was called a defect anyway, twice today, by me. Read the record first.
- **Reaching for `check_fence_interior`.** It walks an expression; `:from` holds a condition.
- **Special-casing past a STOP-1 refusal.** That builds the second grammar the DESIGN rejects.
- **Counting fixtures as corpus adoption.** A test that exercises `accumulate` does not make the
  form used; do not update any "zero uses" claim on that basis.
- **Believing this brief.** Measured once, by one hand, at `07eba8226`, and this document's parent
  FINDING already had its central claim struck within the hour. Re-drive the two-position table
  before leaning on it.
