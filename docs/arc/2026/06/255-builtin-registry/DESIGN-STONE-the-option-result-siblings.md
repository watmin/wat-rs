# DESIGN — STONE: the Option/Result siblings get homes

> **Builder, 2026-08-31:** *"we work on option and result next..."*
>
> The three that remain of the family homed today: `Option/try`, `Result/expect`, `Result/try`.
> `Option/expect`, `Some`, `Ok` and `Err` shipped earlier in the session and are the template.

## ★ THE RULING SPLITS THEM, AND THE SPLIT IS THE FINDING

`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md` asks one question: **does the verb
produce a guaranteed, MATCHABLE outcome?** Read against the three implementations, it separates them:

| verb | what the body does | @Totality |
|---|---|---|
| `Result/expect` | `expect_panic(op, …)` on `Err` — **a raise** | **Partial** |
| `Option/try` | `Err(EvalBreak::Signal(OptionPropagate))` | **Total** |
| `Result/try` | `Err(EvalBreak::Signal(TryPropagate(e)))` | **Total** |

★★ **A propagate SIGNAL is not a raise.** `src/runtime.rs:19458`, verbatim:

> *"`TryPropagate` keeps its legacy behavior: **wrap in the function's own `Err(e)` return**. The type
> checker guarantees this function's declared return type is `(Result :- [_ E])` whenever its body
> contains a `try`, so the wrap is type-correct by construction."*

So `try` yields **"an enum with error-bearing arms adjacent to valid return value"** — the exact shape
the builder's ruling calls total — and the checker *guarantees* it is always matchable. `expect`
yields nothing a caller can match.

★★★ **That is evidence the ruling is well-formed rather than merely stipulated:** applied blind to
three sibling verbs, it blesses the propagation idiom and condemns the panic one — the same `expect`
the builder called *"a fatal mistake… rip them out as we grind forwards"*.

⚠ **`Result/expect`'s `Partial` row is a deliverable, not an embarrassment.** It joins the totality
census that scopes the `expect` purge — 3 rows today, 4 after this stone.

## THE ONE CONTRACT DECISION — pinned

**`try` is TOTAL and `expect` is PARTIAL, and the difference is measured at the body, not inferred
from the family name.** Two verbs sharing a namespace and a suffix-convention get opposite verdicts
because they do opposite things.

## What ships

Three thin `#[wat_intrinsic]` delegates over the existing named fns — `eval_option_try`,
`eval_result_expect`, `eval_try`. Bodies do not move. Their literal dispatch arms
(`runtime.rs:5819,5820,5829`) come out; their three `KNOWN_UNREVIEWED` rows come out (48 → 45).

Arities, from the bodies: `Result/expect` is 2 (`args[0]` value, `args[1]` message); both `try`
verbs are 1.

## ★ A PREDICTION, falsifiable

**All three will trip `checker_skip_debt_is_named_and_frozen`** — measured before briefing: none of
the three carries an `env.register()` TypeScheme, each is checked by a hand-written `check_call` arm.
Same shape as `Option/expect` last time. **Three `FROZEN_CHECKER_DEBT_LEDGER` rows expected (59 → 62).**

⚠ If a row is NOT needed for one of them, the measurement is wrong and that is a finding — not a row
to skip quietly. (`sort$native` was predicted the other way and held: it has a scheme, and needed no
row.)

## Out of scope = REJECTED (not deferred)

- **Retiring `expect`.** The builder's long-term direction, a campaign. This stone makes one more of
  its targets *visible* by declaring it `Partial`; it removes none.
- **The other 45 `KNOWN_UNREVIEWED` verbs.** Named by kind in the seam; this stone takes the three
  whose template shipped today.
- **`:layer`**, untouched and un-guessed.

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **all three, ruled from their bodies** | YES | YES | YES | YES | ✅ **ADMITTED** |
| rule all three `Partial` (family symmetry) | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| home `expect` only; defer the `try` pair | **NO** | YES | YES | — | ⛔ **DISQUALIFIED** |
| rule the `try` pair `Unreviewed` to be safe | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **family-symmetry Honest? NO** — it would put two verbs on the `expect`-purge worklist that do not
  raise, corrupting the census that scopes the campaign.
- **`expect`-only Obvious? NO** — three siblings in one family, one homed and two parked, with
  nothing at the site saying why.
- **`Unreviewed`-to-be-safe Honest? NO** — `Unreviewed` means *nobody looked*. The bodies were read
  and the answer is plain; recording "did not look" about a verb just measured is the exact lie the
  fourth variant exists to prevent.

## Acceptance

| what | command | expected |
|---|---|---|
| the three are registered | `lookup_entry` for each | `Some` |
| ★ the split holds | `metadata-of` `:totality` | `expect` **Partial**, both `try` **Total** |
| the ratchet is satisfied | `KNOWN_UNREVIEWED` | 48 → 45 |
| the prediction holds | `FROZEN_CHECKER_DEBT_LEDGER` | 59 → **62** |
| `Partial` census grew honestly | `@Totality Partial` count | 3 → **4** |
| behaviour unchanged | a `try`-using program, an `expect` on `Err` | as today |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5110/5110, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |
