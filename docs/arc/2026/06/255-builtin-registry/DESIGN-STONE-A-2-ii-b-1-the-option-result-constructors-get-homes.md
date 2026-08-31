# DESIGN — STONE A-2-ii-b-1: the Option/Result constructors get homes

> The verbs `meter-2` made visible. Parked in `KNOWN_UNREVIEWED` by that stone with the note *"NOT
> ruled here … homing `Some`/`Ok`/`Err` is the NEXT stone."* This is it.

## Why

`:wat::core::Some` is the fourth verb blocking the accessor path. With `Option/expect`,
`Record/field-at` and `type` homed (A-2-ii-b-0), a generated record accessor's body still classifies
impure because `(Some self)` denies — measured, and the reason it was never on any worklist is that
the meter could not see it until `meter-2`.

`Ok` and `Err` are its siblings: same shape, same `eval_list` keyword-guard dispatch, same parked
row. Homing one and leaving two is three states where there should be one.

## The rulings — all three from their implementations

`eval_some_ctor` / `eval_ok_ctor` / `eval_err_ctor` (`src/runtime.rs:15023,15047,15071`) are each:
an arity check, then `eval_inner` on the single argument, then a wrap —
`Ok(Value::Option(Arc::new(Some(v))))` / `Ok(Value::Result(Arc::new(Ok(v))))` /
`Ok(Value::Result(Arc::new(Err(v))))`.

| verb | @Purity | @Determinism | @Total | ground |
|---|---|---|---|---|
| `:wat::core::Some` | Pure | Deterministic | **Total** | one `return Err`: the arity check, which retires on homing. The wrap cannot fail |
| `:wat::core::Ok` | Pure | Deterministic | **Total** | same shape |
| `:wat::core::Err` | Pure | Deterministic | **Total** | same shape |

⚠ **`Err` is a constructor, not a failure.** `(Err v)` *builds* a `Result` value; it does not raise.
Under `RULING-a-raise-is-not-an-outcome-…`, a matchable error-bearing arm **is** a total outcome —
`Err` is the shape the ruling calls total, not the shape it calls partial. Do not let the name
suggest otherwise.

## ★ A PREDICTED RED — named in advance, which is the point

Homing these will fire `checker_skip_debt_is_named_and_frozen`, exactly as homing `Option/expect`
did last stone. **Measured before briefing this time:**

- all three are checked by hand-written `check_call` arms (`src/check.rs:4938,4948,4958`);
- **none carries an `env.register()` TypeScheme** — verified, not inferred;
- so `check_env.get` returns `None`, `doc_arg_ret_types_match_checker_scheme` silently skips them,
  and the ratchet demands they be named.

**Disposition: one `FROZEN_CHECKER_DEBT_LEDGER` row each**, with the reason, following the
`Option/expect` precedent set last stone and the `nth`/`reverse` precedent below it. This is the
ledger's designed job — real checking, no scheme to verify the *docs* against.

⚠ I discovered this from a red last stone. Predicting it here is the FM-9 discipline working; if it
does **not** fire, that is a finding about the gate, not a lucky escape.

## What ships

Three thin `#[wat_intrinsic]` delegates over the existing named fns — bodies do not move. Their
`eval_list` keyword-guard arms come out. Three `KNOWN_UNREVIEWED` rows come out. Three
`FROZEN_CHECKER_DEBT_LEDGER` rows go in.

## Out of scope = REJECTED (not deferred)

- **`:wat::core::None`** — `meter-2` excluded it by name with a cited reason: its `eval_list`
  occurrence is a *pattern-clause head* inside `match`'s own implementation, and its real
  expression-position evaluation is an `if`, not a dispatch arm. It already classifies `true` as a
  bare keyword. Not a verb to home here.
- **`sort$native`'s imposition and homing** — A-2-ii-b, which this unblocks.
- **The remaining `KNOWN_UNREVIEWED` population** — 52 rows; three leave here.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **b-1** home all three constructors | YES | YES | YES | YES | ✅ **ADMITTED** |
| home `Some` only (the one that blocks) | **NO** | YES | YES | — | ⛔ **DISQUALIFIED** |
| rule them in `intrinsic_meta`'s hand-list instead | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **`Some`-only Obvious? NO** — three identical constructors in one match, one homed and two parked,
  with nothing at the site saying why.
- **hand-list Honest? NO** — it grows the list the campaign exists to drain, to avoid a registration
  that is available. `meter-2` had to grow it by 7 because those verbs had no home to go to; these
  three do.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ **the accessor finally classifies pure** | `255-probe-the-accessor-classifies-pure.wat` | `true` / `false` (row 1 flips) |
| the three are registered | `lookup_entry` for each | `Some` |
| ratchet satisfied | the three `KNOWN_UNREVIEWED` rows | deleted (52 → 49) |
| the predicted red is disposed | `FROZEN_CHECKER_DEBT_LEDGER` | 3 rows added, each with a reason |
| no widening | probe: effectful fn through a binding | `false` |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5109/5109, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
