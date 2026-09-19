# EXPECTATIONS — a fence-local binder may not shadow a rete variable

Written **before** the strike.

| what | command | expected |
|---|---|---|
| ⭐ **the one real corpus site still compiles** | `where-inline-computed.wat`'s `(let [x ?k] …)`, as a fixture | **compiles and fires.** If this reds, the cure is refusing plain binders |
| the hazard is refused | `(:wat::rete::where (… let [?k "str"] …))` | refused at rule-compile, naming the binder |
| `match` — driven, then decided | the open question | either refused **with a driven demonstration that it binds**, or explicitly left alone with the evidence |
| the variant carries no `fact_type` | read `error.rs` | no `fact_type` field — a fence is not scoped to a fact |
| the retired claim is recorded | `git diff src/rete/validate/typing.rs` | `check_fence_interior`'s doc **rewritten**, not deleted — it currently states the bound this stone retires |
| ⛔ **MUTATION — the refusal is load-bearing** | delete the refusal **in the live source**, re-run | the hazard fixture **REDs**. Restore ⇒ green |
| ⛔ **MUTATION — the guard is load-bearing** | make the refusal fire on ANY binder, not just `?`-prefixed | the **plain-binder** fixture REDs. If it does not, nothing is testing over-rejection |
| nothing else moved | the 27-cell matrix in the SCORE of this stone | every position that checked before still checks |
| floor | `./scripts/floor.sh`, read by you | **0 failed.** Passed ≥ 5523 + the new tests |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

**45–75 minutes.** The source change is small; the fixtures and the `match` question are the work.

## Trap doors

- ⛔ **Refusing plain binders.** `(let [x ?k] …)` is the ONE real corpus use and it is legitimate —
  a local holding a rete variable's value. Refusing it breaks the only site that exists.
- ⛔ **Adding the `match` refusal without driving it.** The `let` hazard is driven; the `match` one
  is my inference from a doc comment. An unverified refusal is a guess with an error message.
- **Descending into the body "while you're there."** Out of scope: it needs a local type env, and
  without one the operands resolve as not-knowable and are skipped anyway. See the DESIGN.
- **Claiming this makes `let` bodies type-checked.** It does not. It retires the JUSTIFICATION for
  not checking them. Say that precisely in the SCORE.
- **Believing this brief.** The corpus counts were measured once, by one hand, with an anchored
  grep — and five instrument errors were made in this session before that anchor was added. Re-run
  the count before leaning on the zero.
