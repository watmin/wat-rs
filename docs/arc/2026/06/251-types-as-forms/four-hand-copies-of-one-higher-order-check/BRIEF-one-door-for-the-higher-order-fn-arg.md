# BRIEF — ONE DOOR for the higher-order fn argument (four hand-copies, one act)

Anchor: `/home/john/work/holon/wat-rs`. Verify with `pwd`. Never use worktrees.
Do not touch `~/work/holon/` (the frozen root).

## Tree state — ⛔ THE FLOOR IS GREEN AND PUSHED. THAT IS THE BASELINE YOU MUST NOT BREAK.

HEAD `58e9563b6`, clean, **0 unpushed**.

```
.floor/2026-09-11T07-11-56Z    5367 run   5367 passed   0 failed   22 skipped     clippy 0
```

**A red start is NOT expected.** Any red you see is yours. This is the first fully green floor of
the campaign; it was 9 failures this morning.

## The defect — not a missing feature, a missing DOOR

`src/collection/infer.rs` has **four** higher-order collection verbs. Each one hand-rolls the
identical check, and no helper is shared:

```
infer_map     :729   OP ":wat::core::map"      expected fn(T) -> U          :761  unify
infer_mapv    :797   OP ":wat::core::mapv"     expected fn(T) -> U          :826  unify
infer_filter  :866   OP ":wat::core::filter"   expected fn(T) -> bool       :898  unify
infer_foldl   :936   OP ":wat::core::foldl"    expected fn(Acc,T) -> Acc    :992  assignable
```

The four blocks are **byte-identical** apart from how `expected_fn_ty` is built:

```rust
if let Some(f_ty) = fn_ty {
    if unify(&f_ty, &expected_fn_ty, subst, env.types()).is_err() {
        local_errors.push(CheckError { span: args[0].span().clone(), kind: CheckErrorKind::TypeMismatch {
            callee: OP.into(),
            param: "#1".into(),
            expected: format_type(&expected_fn_ty),
            got: format_type(&apply_subst(&f_ty, subst))
        }});
    }
}
```

`58e9563b6` changed **one** of them (`foldl`) from `unify` to `assignable`, because unify is
invariant on `Fn` arguments and the `Variant <: Enum` lattice edge was therefore a no-op at every
function slot. **That fix cannot reach the other three, by construction** — there is nothing shared
to fix. `map`, `mapv` and `filter` still refuse a reducer that is correct under the lattice.

⚠ Each verb is ALSO dispatched from two sites (`src/check.rs:2393-2396` and `:4512-4540`).

★ There is no `filterv`, `foldr`, `keep`, `mapcat` or `remove` verb **yet**. That is precisely why
this is cheap now: the fifth copy has not been written. One door means a fifth combinator inherits
the lattice instead of re-deriving it wrongly.

## The strike — TWO STEPS, MEASURED SEPARATELY. Do not merge them.

### STEP 1 — extract the door, change NO behaviour

Add one helper in `src/collection/infer.rs` (shape yours; this is the content):

```rust
fn check_higher_order_fn_arg(
    op: &str,
    fn_ty: Option<&TypeExpr>,
    expected_fn_ty: &TypeExpr,
    span: &Span,
    subst: &mut Subst,
    env: &CheckEnv,
    local_errors: &mut Vec<CheckError>,
)
```

Route all four call sites through it. **In step 1 each caller keeps the comparator it has today**
(`foldl` → `assignable`; the other three → `unify`) — pass it in, or land step 1 with the helper
taking a flag you delete in step 2. Whichever is cleaner; the requirement is only that step 1 is a
**pure refactor**.

**Step 1's acceptance is that NOTHING moves:** `5367 run / 5367 passed / 0 failed`, and no `.edn`
golden is recaptured. If any number or golden changes, step 1 was not a refactor — STOP and report.

### STEP 2 — one comparator: `assignable` for all four

Delete the flag. All four verbs compare with `assignable`.

## ⛔ STOP triggers — each is a REJECTION, report and halt

**STOP-1 — `map`'s output type is inferred BY the comparison.** `infer_map` builds
`expected_fn_ty` with a FRESH var as the return (`u_var`, `:756`), then reads the answer back out
with `apply_subst(&u_var, subst)` (`:781`) to type `Stream<U>`. `unify` binds `u_var` as a side
effect. **`assignable` must bind it the same way or `map`'s return type silently becomes a fresh
var** — the exact "accepted but never typed" defect arc 251 just spent two stones killing. Prove
`U` is still concrete: a `map` whose result feeds a typed slot must still check. If it does not,
STOP — the helper may need to keep unify's binding behaviour in the return position.

**STOP-2 — contravariance on an unresolved element type.** The new `Fn` arm
(`src/check.rs:17168`) compares arguments CONTRAVARIANTLY: `assignable(exp_arg, got_arg)`. In
`map`/`filter`, `elem_ty` can itself be a fresh `Var` from `extract_lazyable_elem`. A contravariant
`assignable` on a Var may bind in the **opposite** direction from the unify it replaces. If any
collection test changes behaviour, STOP with the verbatim block.

**STOP-3 — a golden that pins an error MESSAGE.** `expected`/`got` render through `format_type`.
Loosening the comparator changes WHICH mismatches are reported at all. Any `.edn` golden that moves
is a finding, not a recapture — STOP and report which.

**STOP-4 — if you find a fifth copy of this block** anywhere outside these four functions, STOP and
name it. The census above is mine and it is `grep`-derived; a fifth site means the door has to be
placed differently.

## Acceptance

```
1. ONE helper; all four verbs call it; no copy of the block remains
2. step 1 floor  5367 / 5367 / 0     unchanged, no golden recaptured
3. step 2 floor  5367 / 5367 / 0     plus the new probe below
4. clippy --release --all-targets -- -D warnings   0
```

**5. A POSITIVE probe, and it must FAIL before step 2 and PASS after** — otherwise step 2 is
unfalsifiable. For EACH of `map`, `mapv`, `filter`: a collection of a VARIANT element type passed
to a function declared over the PARENT ENUM is ACCEPTED. Shape:

```wat
(:wat::core::defenum :u::Op :wat::enum::Pure :Mark [] :Other [])
(:wat::core::defn :u::takes-op [o <- :u::Op] -> :wat::core::i64 1)
;; a Vector of Op.Mark, mapped by a fn over Op — REFUSED today, must be ACCEPTED after step 2
```

**6. The NEGATIVE control must survive**: a function over the VARIANT must still be REFUSED for a
collection of the ENUM (narrowing stays one-way). Reuse the shape in
`tests/types/probe_arc251_enrol_the_variant_in_the_lattice__fn_narrow_param_for_wide_slot.wat`.

⛔ Assert refusals with a STRUCTURAL `.edn` golden captured via `UPDATE_EDN`, never
`.contains("TypeMismatch")` — `tests/lint/no_loose_string_assert.rs` fails the floor on that, and it
has caught seven such sites in the last two stones.

## Tier

You edit, you write the probes, you REPORT. **Do NOT commit. Do NOT run `scripts/floor.sh` or
clippy** — the orchestrator measures centrally, once, on a quiescent tree.

⛔ Do NOT call `pulsare_yield` or contact any peer. Report to the orchestrator only.
