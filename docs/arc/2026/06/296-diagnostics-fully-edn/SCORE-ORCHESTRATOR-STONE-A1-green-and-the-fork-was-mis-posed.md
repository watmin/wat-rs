# SCORE (orchestrator) — STONE A-1: GREEN, and the fork I put to the builder was MIS-POSED

**Independent re-run.** Floor and clippy central.

```
FLOOR EXIT=0    Summary [ 191.548s] 5271 tests run: 5271 passed, 21 skipped
CLIPPY          the same 5 pre-existing dead_code items. No new ones.
```

## What landed, verified by reading

```
contains_type_var        src/declare/typevar.rs — a VISITOR over walk_type_expr, which gained a
                         visit_var callback. `fn walk_` stayed 2. STOP-4 held.
join_if_branches         `if` — the form's type is the one BOTH branches can be seen as.
                         then <: else -> else; else <: then -> then. NOT then-authoritative.
relate_value_to_slot     send AND try-send — one helper, not a third rule.
grep -c is_subtype       30, unchanged. `assignable` reused, no new arm. STOP-3 held.
```

★ **The rider found a third site my brief missed.** I named `send`; it found `try-send` shares the
same `I` slot (its own comment says so) and routed it through the same helper rather than writing a
third rule. That is the stone's own thesis applied to a site the brief did not know about.

## ⛔⛔ THE CORRECTION — I asked for a ruling on a fork that does not exist

I told the builder that *"always subsume"* removes variable solving, and built the entire
four-questions comparison on it. **I never read `assignable`'s last line:**

```rust
unify(actual, expected, subst, types).is_ok()
```

`assignable` tries its subtyping arms and then **falls through to unification.** So the
unconditional shape never lost inference. Measured with a sabotage forcing the subsume path, across
three separately-constructed fixtures:

```
if_still_solves_a_type_var             normal=0   subsume-always=0
(the `if` is the only thing pinning T) normal=0   subsume-always=0
(the receiving slot demands the arg)   normal=0   subsume-always=0
```

★★★ **The capability guard cannot fail, because the capability was never at risk.** EXPECTATIONS
row 4 said in advance: *"If it fails silently… the row is not doing its job and that is itself a
finding."* It was a finding, and I only reached it because the row demanded proof it COULD fail.

`[[feedback_four_questions_cannot_see_a_shared_premise]]` — the four questions discriminated between
options and never validated the premise BOTH rested on.
`[[feedback_a_real_fork_can_still_be_mis_posed]]`.

The false rationale is **corrected in place in the probe header before landing**, with the original
sentence quoted and marked refuted — a reader who believed it would credit that file with catching
something it cannot catch.

## ⚠ What genuinely remains open — hygiene, not acceptance

`assignable`'s arms call `unify` internally on the way through, so a FAILED arm can leave partial
bindings in `subst` before the fall-through. The conditional rule never enters those arms when a
variable is present. **That difference is UNMEASURED**, and it is the only surviving argument for
the conditional shape over calling `assignable` directly.

⬜ Not resolved here, and named rather than quietly kept: whether `contains_type_var` and the new
`visit_var` callback earn their place, or whether the stone collapses to a bare `assignable` call.
"No observable difference today" is how a silent binding leak survives, so this wants a measurement,
not a shrug.

## Rows

| # | expected | actual |
|---|---|---|
| 1 | subtype-related `if` EXIT 0 | ✓ (was 1) |
| 2 | parameter control EXIT 0 | ✓ |
| 3 | ⛔ unrelated branches EXIT 1 | ✓ |
| 4 | ⛔⛔ capability guard EXIT 0 | ✓ — **and proven VACUOUS by sabotage** |
| 5 | `test(a1_one_rule)` 4 passed, 0 skipped | ✓ (5 with the corrected header's own row) |
| 6–8 | P-1 / P-2prereq / P-3 unmoved | ✓ |
| 9 | no second subtyping test | ✓ `is_subtype` count 30 |
| 10 | no fifth walker | ✓ `fn walk_` 2 |
| 11 | both positions agree | ✓ same pair accepted at parameter and at `if` |
| — | FLOOR | ✓ **5271/5271** |

All five STOPs held. The `if` symmetry was checked by swapping the branches, not assumed.

## Disposition

**LANDED.** The behaviour is right, the floor is green, and the one thing that was wrong — my
rationale — is corrected on disk before the push rather than after.
