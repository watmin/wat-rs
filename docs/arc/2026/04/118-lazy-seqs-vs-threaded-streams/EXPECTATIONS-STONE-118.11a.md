# EXPECTATIONS — 118.11a · mint `next` + `NextOutcome`

**Written before the strike, 2026-08-17, against `428b49c6`** (floor 4703/4703, clippy 0,
ignores 13). Fixed here so the result cannot move the goalposts.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 0 | ★ the verb does not exist yet | call `(:wat::stream::next …)` pre-edit | **unknown-function**, captured verbatim |
| 1 | `Item` carries the head | kept test | `value` = first element |
| 2 | empty → named end | kept test | `NextOutcome::Exhausted` |
| 3 | ★★ **one force per call** | printing `f`, one `next` on `(map f v)` | **exactly 1 line** |
| 4 | `rest` advances | `next` on row 1's `rest` | second element |
| 5 | the memo is untouched | `git diff src/stream/mod.rs` | no change to `forced` |
| 6 | existing verbs unmoved | floor | 0 FAIL |
| 7 | test is KEPT | `git status --short tests/` | added, not deleted |
| 8 | floor | `scripts/floor.sh` → Summary | **≥4704 passed, 0 failed, 0 timed out** |
| 9 | clippy | `cargo clippy --release --all-targets` | **0** |
| 10 | ignores | the `#[ignore]` grep | **13** |

Row 8 baseline: `428b49c6` ran **4703**. The kept test adds at least one → **4704+**.

## Independent prediction

**Runtime: 40–70 minutes.** Two release builds dominate. The edit is small but touches three
unfamiliar registries — the type registry, the dispatch table, and the checker — and the rider will
need to read `RecvOutcome`'s declaration carefully before writing `NextOutcome`'s.

**Time-box: 140 minutes.**

## Trap doors — named before the strike

1. **★ Parametric enum registration.** `NextOutcome<T>` must be parametric. `check.rs:1977-1982`
   already warns about this exact class: *"a nullary variant infers as the un-parametrized
   `RecvOutcome` and fails to unify with the `RecvOutcome<Response>` a use site expects."* **The
   `Exhausted` nullary variant is precisely that hazard.** Expect it; row 2 catches it.
2. **`TypeExpr::Var` for the scheme's type parameter.** Wrong constructor — `Var(u64)` is a
   synthetic unification variable. This exact mistake cost a correction in 279.3. Use
   `TypeExpr::Path("T".into())`.
3. **Writing a second forcing loop** instead of reusing `realize`. Two forcing paths is how the
   three-call protocol got here in the first place.
4. **"Fixing" row 3 with a cache.** If `next` forces more than once, the fix is to force less, not
   to memoize more. STOP-1.
5. **Row 0 skipped.** Then rows 1–4 could pass on a stone that changed nothing.

## What would make me call this Mode B

- Row 3 green but achieved by consulting the existing memo — that ships the number while leaving the
  primitive dependent on the thing stone B deletes.
- Any existing verb's behaviour moving. This stone is additive; row 6 is not negotiable.
- The memo touched at all (row 5).
- Any `#[ignore]` added.

## What I will re-run myself before committing

Rows 1, 2, 3, 4, 8, 9, 10 — independently, on my own invocation, per FM 9. **Row 3 especially**: it
is the entire point of the stone, and it is the one row where a plausible-looking implementation can
be wrong in a way the other rows do not catch.

## What this stone is NOT allowed to claim when it lands

That the memory defect is fixed. **The memo is still in place**; memory is unchanged by design, and
row 6 requires it. O(1) is stone B's acceptance test and remains a **prediction** — the last
prediction in this area (that removing the memo alone reaches O(1)) was wrong.
