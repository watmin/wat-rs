# DESIGN — STONE 1a-β-0b: a form that never evaluates gets a purity pole of its own

> **Builder, 2026-09-02:** *"prefixes declaring properties die when the registry matures..... each
> name... gets their own declarations..... the prefix is nothing but a namespace......"*
>
> *"as for the value..... it may be proper to add another variant?......"*
>
> The first is **doctrine**, recorded below and larger than this stone. The second is the question
> this stone answers, and the measurement says **YES**.

## The blocker, one line

`:wat::core::defsurface` is consumed whole at freeze and **never reaches evaluation**. `@Purity` has
three poles — `Pure`, `Effectful`, `Preserving` — and all three are wrong for it. The floor is red at
`declared_purity_vs_effectful_by_prefix_census`. Full measurement:
`[[NOTE-the-prefix-guess-has-run-out-of-road]]`.

## ★★★ THE DECIDING MEASUREMENT — every consumer of `@Purity` asks a RUNTIME question

I read all four, rather than reasoning from the axis's name:

```
src/rete/purity.rs:474     pure: matches!(e.purity, Pure | Preserving)     may this appear in a RULE BODY
src/rete/purity.rs:2109    is_effectful_op: matches!(e.purity, Effectful)  does CALLING it have an effect
src/intrinsic/mod.rs:2188  purity_mandated_examples: Pure|Preserving       demands a RUNNABLE @example
src/intrinsic/reflect.rs:84  pure = matches!(Pure | Preserving)            the doc surface's claim
```

**All four are claims about evaluating the verb.** So for a form with no runtime existence:

- `Pure` says *"evaluating this is safe"* — it cannot be evaluated at all, and the mandate would then
  demand a **runnable** `@example` that provably raises. A false doc claim.
- `Effectful` says *"evaluating this has an observable effect"* — there is no evaluation to have one.
- `Preserving` says *"my purity is my sub-forms' purity"* — `:features` is a static member list, never
  evaluated. Nothing to inherit from.

★ Three poles, three different false statements. **That is what a missing pole looks like** — not an
awkward row.

## THE ONE CONTRACT DECISION — pinned

**`:Unevaluated` — a fourth `Purity` pole meaning: this form is never evaluated, so the axis has no
runtime verdict to give.**

It sits beside `Preserving` as the *other* way a special form has no purity of its own — `Preserving`
because it inherits one, `Unevaluated` because there is no evaluation. ⚠ **The name states the
FORM's condition, never a moment.** `:FreezeTime` was considered and rejected: `runtime-meta.wat`'s
own axis discipline is *"the DOING, not the moment it happens"*, and a moment-named pole would import
`Category`'s job into `Purity`. `:Declarative` rejected for the same reason — that word is
`Category::Declaration`'s.

## ★★ Why this is SAFE, measured rather than hoped

Every consumer above is a `matches!` on the poles it accepts. **A new variant matches none of them,
so each already computes the correct answer with no edit:**

| consumer | answer for `Unevaluated` | is that right? |
|---|---|---|
| rete fence `pure` | `false` → refused in a rule body | ✅ it is not a runtime expression at all |
| `is_effectful_op` | `false` → not treated as effectful | ✅ there is no call to have an effect |
| `purity_mandated_examples` | not pure-and-det → no runnable example demanded | ✅ the exact trap that made `Pure` dishonest |
| the census's assertion | `declared_effectful` false → does not fire | ✅ the red dissolves, without weakening the gate |

★★★ **And the sweep is COMPILER-ENFORCED.** `wat_intrinsic.rs:908` and `wat_special_form.rs:124`
both `match doc.purity` **exhaustively** — adding a pole breaks the build at exactly the two sites
that must acknowledge it. No census of mine is load-bearing here
(`[[feedback_impose_the_check_and_read_the_screams]]`).

## ⛔ THE GATE — or the pole becomes a dumping ground

A pole meaning *"never evaluated"* is a claim about the row that the row could simply be wrong about,
and its whole value is that four consumers trust it. So it is made **structurally checkable**:

> A row declaring `@Purity Unevaluated` must have **no `handler`, no `tail_handler`, and no `Eval` or
> `Tail` impl role.** Any of the four is a route to evaluation, and the claim is false.

Derived from the row's own registration facts — not a name list. `[[extirpare]]`'s check rung, and
the same shape as *"a registered row that carries a handler may not also have a literal arm."*

⚠ It cannot reach a hand-written `runtime.rs` match arm, and the design says so rather than implying
coverage it lacks: this gate refuses the three registry-visible routes. Naming the fourth is
`[[feedback_a_containment_argument_must_name_its_consumers]]` paid up front.

## What this stone does NOT do — affirmatively, not deferred

- **`Determinism` and `Totality` keep describing the freeze-time pass**, and `defsurface` keeps the
  rider's `Deterministic` / `Partial` with its grounds. They are TRUE statements about the pass that
  does happen, nothing reads them as runtime safety claims, and the fence already refuses the verb on
  `pure = false` alone. ⚠ Written down so a future self does not "fix" them into symmetry with a
  ruling nobody made.
- **`effectful_by_prefix` is not retired here.** The builder's doctrine says prefixes declaring
  properties die when the registry matures; the registry has not matured — 19 of the five hand-lists'
  20 entries are still unregistered. Retiring the prefix today makes an unregistered verb default to
  *not* effectful, a silent widening of what the fence admits. **That is Phase 3c and it needs the
  census at 0 first.**
- **The other eight declaration forms.** Still 1a-β.

## THE FOUR QUESTIONS

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **`:Unevaluated` pole + the structural gate** | YES | YES | YES | YES | ✅ **PICKED** |
| keep `Effectful`, exempt `Declaration` from the census | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| declare it `Pure` | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| retire `effectful_by_prefix` now | YES | YES | **NO** | — | ⛔ DISQUALIFIED |
| `:Unevaluated` pole, no gate | YES | YES | YES | **NO** | ⛔ DISQUALIFIED |

- **exempt-the-category Honest? NO** — `:wat::core::def` is also `Category::Declaration` and has five
  real `runtime.rs` arms. The category does not mean "never evaluates"; the exemption would lie about
  `def`.
- **`Pure` Honest? NO** — measured: it triggers the runnable-example mandate on a form that cannot be
  run. The rider found this and refused it, and was right.
- **retire-the-prefix Honest? NO** — see above; it flips the default for every unregistered verb.
- **no-gate Good UX? NO** — four consumers trust the pole. An unfalsifiable claim they trust is the
  thing this arc exists to eliminate.
- **Simple? YES**, and unusually so: **zero consumer edits.** The pole's correct behaviour at all four
  sites is what an unmatched variant already produces.

## Blast radius

```
wat/runtime-meta.wat                      + :Unevaluated with its ;; prose  ← the source of truth
crates/wat-macros/wat_intrinsic.rs        + one match arm (compiler-forced)
crates/wat-macros/wat_special_form.rs     + one match arm (compiler-forced)
src/intrinsic/mod.rs                      + the structural gate
src/intrinsic/special/defsurface.rs       @Purity Effectful → Unevaluated, ground rewritten
```

⚠ The `.wat` edit is ONE form in ONE file — a variant addition, not a structural corpus migration —
so the wat-fix codemod doctrine does not apply. Stated because the reflex is right and the exception
should be argued, not assumed.

## Acceptance — rows chosen to be unfakeable

| what | command | expected |
|---|---|---|
| the pole exists in the SOURCE OF TRUTH | `wat/runtime-meta.wat` | `:Unevaluated` with prose; Rust follows |
| the red is gone | the census test | green — and its 121 count is now **120** |
| ⛔ the census did NOT weaken | re-declare a `:wat::kernel::` row `Effectful` with the prefix stripped | still RED |
| ⛔ the gate can FAIL | give `defsurface` a `role = eval` annotation | RED, naming it |
| ⛔ the gate is not vacuous | it inspects ≥ 1 row and names which | `defsurface` |
| the fence refuses it | `:wat::rete::pure?` on `:wat::core::defsurface` | false |
| ⛔ nothing else moved | the 120 remaining census entries | the same names as before |
| floor | `scripts/floor.sh`, exit UNPIPED | 5121/5121, 0 failed |
| clippy | `-D warnings --all-targets` | 0 |

## ★ THE DOCTRINE, recorded because it outlives this stone

> **"Prefixes declaring properties die when the registry matures. Each name gets its own
> declarations. The prefix is nothing but a namespace."**

That is the RULING's item 6 sharpened, and it names the end state for `effectful_by_prefix` (8
prefixes), `is_reserved_prefix` (the founding target), and `check.rs`'s
`:wat::kernel::`/`:wat::std::` arity-door guard. **A namespace is an address, never a claim.** This
stone does not execute it; it stops making the problem worse, by giving one name a declaration a
prefix could never have made.
