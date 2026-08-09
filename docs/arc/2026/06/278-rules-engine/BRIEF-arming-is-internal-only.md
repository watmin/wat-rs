# BRIEF — arming an alarm is INTERNAL-OPS-ONLY, refused at the definition site

> **Builder-ruled 2026-08-09: "let's do (a) - build the checker rule."** The four-questions verdict and
> the run that forced it are in `DESIGN-STONE-the-call-context.md`; do not re-derive them.

## The defect, proven by run — this is live TODAY, not hypothetical

A `defservice` handler can arm a **public** (client-facing) op via an `Alarm`. When the timer fires,
the handler executes with a **timer** in the `idx` slot — no client — mutates durable state, returns
`Outcome::Reply`, and **the reply goes nowhere with nothing reported.** Measured:

| # | form | result |
|---|---|---|
| 1 | `:op :poll` (bare keyword) | refused, exit 1 — but only by ACCIDENT (below) |
| 2 | `:op (:probe::tick2::Op::Bump (…Request…))` explicit ctor | **`--check` exit 0 — ACCEPTED** |
| 3 | run of #2, with a state-mutating witness | **fired**: durable count `7 → 8`, exit 0, no error |

**#1 is not a wall.** The macro's keyword→`Op` rewrite covers *internal* (`-`) ops only
(`wat/service.wat:992`), so a bare `:poll` never becomes an `Op` and an `Alarm<keyword>` /
`Alarm<Op>` mismatch trips incidentally. The explicit ctor is the route around, and it is the natural
form if you follow the types.

This is a **silent discard** — the class R55/R57 spent themselves annihilating.

## ★ THE DISCRIMINATOR ALREADY EXISTS — this is a structural property, not a heuristic

`wat/service.wat:876-892` and `:1013-1020`: an internal op's `Op` variant **retains its leading dash**,
deliberately and scope-preserved —

```clojure
;; `:-Pascal []` (nullary). Dash preserved SCOPED here (strip `-`, kebab->pascal, re-prepend `-`)
variant-pascal (:wat::core::if is-internal
                 (:wat::core::string::concat "-" (:wat::core::string::kebab->pascal-in surface-kw …))
                 (:wat::core::string::kebab->pascal-in surface-kw op-str))
```

So the `Op` enum carries variants like `Poll`, `Bump`, **`-Tick`**. **A variant whose name starts with
`-` is internal; one that does not is public.** The rule needs no new metadata — only this name.

## The rule

**In an `:wat::service::Alarm`'s `op` position, a constructed `<service>::Op` variant whose name does
NOT start with `-` is a CHECK ERROR at the construction site.**

Rationale in one line: an alarm has no client, so only an op that is *declared* to have no client may
be armed. Internal ops already are exactly that — their arm is 1-param `[s]` and returning `Reply`
from one is already a located assertion (`service.wat:1063-1073`). This closes the same gap one step
earlier, at compile time.

## Read in order

1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-the-call-context.md` — the § "RUN 2026-08-09" block
   (the three results above, with the reproduction forms).
2. `wat/service.wat:56` — `(defrecord :wat::service::Alarm<O> [after <- Duration  op <- :O])`, the subject.
3. `wat/service.wat:876-892` + `:1013-1020` — the dash-preserved variant naming. **This is the rule's
   ground truth. Read it before writing the predicate.**
4. `src/check.rs:12032` — `infer_kwargs_construct_check`, the **kwargs** ctor path
   (`(:wat::service::Alarm :after … :op …)` — the form the exemplar and all real code use).
5. `src/check.rs:11924` — `infer_aggregate_new_check`, the **positional** ctor path. See STOP-1.
6. `tests/services/probe_arc278_self_scheduling.wat:45` — the GREEN case that MUST keep working
   (`:op :-tick`). It is the non-vacuity control for the whole strike.

## Implementation sketch

At each ctor path, when the aggregate being constructed is `:wat::service::Alarm`, inspect the
argument landing in the `op` field:

```rust
// If the op arg is a literal enum-variant constructor call, read its VARIANT name.
// Internal ⇔ the variant name begins with '-' (wat/service.wat:876-892).
// A public variant here is a compile error: an alarm has no client to reply to.
```

- Emit a **new `CheckErrorKind`** carrying the offending variant name and the service's `Op` type.
  The message must say *what* is wrong, *why* (an alarm fires with no client), and *what to do*
  (declare an internal `-` op and arm that; a public op and a `-tick` may share one helper fn).
- **Name the error kind descriptively for now and mark an intueri cast OWED in your report** — do not
  block the strike on a ward, and do not treat your placeholder as ratified.

## ⛔ STOP triggers

1. **STOP-1 — COVER BOTH CTOR PATHS.** If you gate only the kwargs form, the positional form is the
   route around — and that is *exactly* how the existing accidental refusal was bypassed (result #1 vs
   #2 above). A rule with a second door is not a rule. If the positional path cannot reach the field
   name for a structural reason, **STOP and report it** rather than shipping half.
2. **STOP-2 — the DYNAMIC case is out of scope, and say so.** If the `op` argument is not a literal
   variant ctor (a variable, a call result), the checker cannot decide it. **Do not guess, do not
   refuse conservatively, do not silently pass while implying coverage.** Handle the literal case,
   and state the limit plainly in your report and in a code comment. (It is a narrow limit: a handler
   receives `req`, never an `Op`, so a literal ctor is realistically the only way to obtain one.)
3. **STOP-3 — the internal case MUST STILL WORK.** `tests/services/probe_arc278_self_scheduling.wat`
   arms `:op :-tick` and must stay green, both loci. If it reddens, the predicate is inverted or too
   broad — STOP.
4. **STOP-4 — do NOT change the `Alarm` record, the `Outcome` enum, or the serve loop.** This strike
   is a checker refusal only. Reshaping the type (e.g. a separate `InternalOp` enum) was considered
   and is NOT this stone — it collides with the timer's message type reaching `poll'` as an `Op`.
5. **STOP-5 — if the floor moves off 4378/0, STOP** and report the failing test's whole block verbatim
   plus the exact assertion. Expect zero corpus impact: nothing in the tree arms a public op.

## The acceptance gate — build it, both directions

`tests/services/probe_arc278_arming_is_internal_only.{rs,wat.bad}` + `_control.wat`, modelled on
`tests/types/probe_arc278_opaque_purity_wall.*` (landed this session — copy its shape):

- **`.wat.bad`** — a `defservice` whose handler arms a **public** op via the explicit ctor
  (`:op (:probe::…::Op::Bump (…Request…))`). MUST fail to load with the new error kind, and the
  diagnostic MUST name the offending variant.
- **`_control.wat`** — byte-identical but arming an internal `:-tick`. MUST load. Without it the RED
  proves "something in the fixture is bad", not "exactly the public-op arming is refused"
  (R59 `NISI FRANGAS, NIHIL PROBAS`).
- The reproduction forms are in the stone's § "RUN 2026-08-09" — copy them; they are known to
  `--check` clean today, which is what makes them a valid RED gate.

## Weigh

`cargo build --release` → `cargo nextest run --release -E 'test(arming_is_internal_only) or test(self_scheduling)'`
→ `./scripts/floor.sh` (read the **Summary line**, expect **4378 passed / 0 failed** + your 2 new
tests = 4380) → `cargo clippy --release --all-targets` → 0.
