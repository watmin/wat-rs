# BRIEF — S5's last form: mirror `fn`

Closes **#56 (S5)**. Spec: `DESIGN-STONE-where-admits-only-rete-ops.md` § "✅ CORRECTED 2026-08-02 —
`fn` IS mirrorable, and the structural guards BYPASS the fence" — read that section first; it
carries the retraction, the grounding this brief rests on, and the builder-ratified ruling below.

**★ WHY THIS IS NOT OPTIONAL — ratified 2026-08-02.** *Everything inside a `where` is
rete-namespaced.* A complete DSL is closed over its own vocabulary; a `where` reaching into
`:wat::core::` for its syntax is not a DSL but wat with a list of restrictions. So the rete `fn`
name must EXIST before #57 can arm the structural arms to require it. This mirror is a prerequisite
for arming, not a cosmetic.

**Tree state:** HEAD `48b62a76`. Floor 4315/4315/0/262, rete 237/0/9, lint 66/0, clippy zero,
corpus 9 pairs / 98 rows. #59 has just added `and`/`or`/`ann-form` arms to `eval_tail` — **that is
unrelated to you; `fn` never tail-calls and gets no `eval_tail` arm** (STOP-4).

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it. Your turn ends when the numbers are in your hands.

## The work, in one paragraph

Mint `:wat::rete::core::fn` so an anonymous lambda can be written inside a `where` with the rete
spelling. It is the **same shape `match` took in #56** — one inference route, one structural-guard
widening, one table row — and the runtime needs nothing.

Target form:

```clojure
(:wat::rete::core::fn [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::rete::i64::+ 0 x :undefined -1))
```

## Already ground — take as given, do not re-derive

- **An anonymous `fn` is expressible as a free value today.** Proven by run: passed straight to
  `foldl`, never bound, never in a `def`. There is no "must be bound to a symbol" rule.
- **`crate::function::infer_fn` is a clean helper** (`check.rs`'s `":wat::core::fn"` arm calls it and
  nothing else), exactly like `infer_if`/`infer_let`/`infer_match`.
- **The runtime is free.** `dispatch_rete_op`'s `Alias | Form` arm re-dispatches on `core_name`, and
  `dispatch_keyword_head_value` has a `":wat::core::fn" => crate::function::eval_fn` arm. A `fn` form
  never tail-calls, so `eval_tail` is NOT involved — do not add an arm there.
- **The three "blocker" sites are narrow and documented** (the stone's table): the def-shape parser
  is about `(def :name (fn …))` and irrelevant here; `is_fn_form_expr` is duplicate-diagnostic
  suppression; the two arc-212 walkers are scope-boundary stops on the process-spawn path. **None
  blocks this. Do not try to fix them** — they are task #60.

## Read these rooms, in order

1. **`git show ec958a40`** — #56, which mirrored `match` by exactly this recipe. **This is your
   exemplar; read it before anything else.**
2. `src/rete/vocabulary.rs` — THE ONE TABLE and its `Form`-class doc (which currently says `fn` has
   no row "as of #56" — update that sentence).
3. `src/check.rs` — `infer_rete_form`, and its STOP-3 doc note about `fn`. **That note is now
   superseded; rewrite it rather than leaving a retracted claim in the source.**
4. `src/check.rs` — the `":wat::core::fn"` arm (`grep -n '^            ":wat::core::fn" =>'`).
5. `src/rete/purity.rs:775` — the `fn` structural guard, and `:745` for how `match`'s was widened.

## Implementation sketch — three edits, mirroring `match`

1. **`vocabulary.rs`** — one row: `rete_name: ":wat::rete::core::fn"`, `core_name:
   ":wat::core::fn"`, `class: Form`, `params: &[]`. For `meta`, mirror what `purity.rs` answers for
   core `fn` — and note (as `match`'s row does) that `meta` is largely vestigial for a structural
   row, because `classify_expr` decides before `head_ok` is reached.
2. **`check.rs`** — one arm in `infer_rete_form`: `":wat::core::fn" =>
   crate::function::infer_fn(...)`. Match the neighbouring arms' shape exactly.
3. **`purity.rs:775`** — widen the guard through `resolve_core_name`, **exactly** as `:745` was
   widened for `match`. ONE indirection, never a duplicated arm body.

## STOP triggers — rejection criteria. Ship nothing, report the gap.

1. **STOP-1 — `infer_fn`'s signature does not match its siblings.** The other `infer_rete_form` arms
   take `(args, head_span, env, locals, fresh, subst)`; `infer_fn` may differ (it appears to take
   `(args, env, locals, fresh, subst)` — no `head_span`). If adapting it needs more than passing the
   arguments it wants, STOP and report rather than reshaping a shared helper.
2. **STOP-2 — a second table.** A rete op named anywhere but `vocabulary.rs`.
3. **STOP-3 — the widening needs a duplicated arm.** If `resolve_core_name` cannot be used the way
   `match`'s guard uses it, halt and report; do not copy the arm body.
4. **STOP-4 — scope.** Do NOT touch `try_parse_fn_shape_def`, `is_fn_form_expr`, or the arc-212
   walkers (task #60). Do NOT add an `eval_tail` arm. Do NOT arm the fence. Do NOT touch
   `wat/rete.wat`.
5. **STOP-5 — the `_` wildcard on an enum scrutinee is doctrine-illegal.** Name every variant.
6. **STOP-6 — the corpus moves.** `check-where-shapes.sh` must stay 9 pairs / 98 rows agreeing.

## Scorecard — report each row's real result

| # | what | expected |
|---|---|---|
| 1 | `cargo build --release --all-targets` | exit 0, **zero** warnings |
| 2 | `cargo clippy --release --all-targets` | exit 0, **zero** warnings |
| 3 | ★ a rete `fn` type-checks and evaluates as a value | the builder's target form above, run |
| 4 | ★ its BODY is fence-checked | a rete `fn` with an impure body classifies NOT pure |
| 5 | ★ control for row 4 | the same shape with a pure body classifies pure |
| 6 | ★ the structural guard fires (pattern/params not walked as exprs) | mirrors `match`'s gate |
| 7 | ★★ row 6 goes RED without the widening | revert the guard to literal-only, watch it die, restore — **report both observations** |
| 8 | `cargo test --release --test rete` | ≥ **237** passed, 0 failed (baseline at `48b62a76`) |
| 9 | `cargo test --release --test lint` | 66 passed, 0 failed |
| 10 | corpus unmoved | 9 pairs, 98 rows agreeing |
| 11 | fence still unarmed | `wat/rete.wat` untouched |
| 12 | no `rune:lint` added | zero |
| 13 | the superseded STOP-3 note is rewritten | no retracted claim left in `check.rs` |

**Row 7 is the one that decides this shipped.** Rows 4+5 together, not separately — a body-check
test that only shows the impure case proves nothing about the pure one.

**Do NOT run `cargo nextest run`** — the orchestrator weighs the floor centrally, once, after your
tree is quiescent.

## Two lint traps that have bitten twice in this arc

A doc comment or assert message that **parses as a wat list** trips `no_inlined_wat_in_tests`; a
`contains(...)` on a rendered error trips `no_loose_string_assert` (match the typed
`RuntimeErrorKind`). Fix at the root — **no `rune:lint`**.

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not add `#[allow(dead_code)]`
to silence a signal.
