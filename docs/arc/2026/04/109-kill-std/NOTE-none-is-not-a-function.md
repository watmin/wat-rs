# NOTE — `None` is not a function

**Filed here because arc 109 owns the `None` grammar.** `src/check.rs:2422` (branch `sns-sqs`):
*"Arc 109 slice 1h: bare `:None` is a retiring grammar exception."*

**Found 2026-09-04 on `sns-sqs`, during arc 278.** Recorded, not fixed — see *The ruling* below.

## THE FLAW

**`(:wat::core::None <anything>)` type-checks for a non-primitive type keyword and raises
`UnknownFunction` at runtime.**

Measured, this branch:

| form | result |
|---|---|
| `:wat::core::None` | **`":None"`** — the only correct spelling |
| `(:wat::core::None :- [:wat::core::String])` | `RuntimeError` — *unknown function `:wat::core::None`* |
| `(:wat::core::None :wat::core::String)` | `CheckError` — *"`:wat::core::String` is a TYPE keyword, not a value"* (Doctrine 1, arc 242) |
| `(:wat::core::None :wat::WatAST)` | ⛔ **type-checks, then `RuntimeError` — unknown function** |

`None` is a **keyword**, not a callable. `check.rs:2415-2424` handles it as *"nullary constructor of
the built-in `(:Option :- [T])` enum … infers as `(:Option :- [T])` with a fresh `T`; unification
against the expected type sharpens `T` at the use site."* The `T` comes from **context**. There is no
argument to give it, and no application form for it.

## WHY IT SURVIVES THE CHECKER

**Doctrine 1 (arc 242) rejects a type keyword in value position — but only for the primitives.**
`:wat::core::String` and `:wat::core::i64` are caught. `:wat::WatAST`, `:some::Enum::Reply`, and
other type keywords are not, so `(None <that>)` passes the checker and detonates on evaluation.

The gap is between *"this looks like a call"* and *"there is nothing here to call."* `cernere`'s
class exactly: **a phantom form** — one that looks valid, traces to no definition, and is caught by
neither the grammar nor the type checker.

## THE CORPUS SITES (a second one landed — see the 2026-09-05 section below)

**`wat-scripts/fixes/positional-to-kwargs.wat:27`**

```wat
(:wat::core::defn :user::fieldvec-at [ch <- (:wat::core::Vector :- [:wat::WatAST]) i <- :wat::core::i64]
  -> (:wat::core::Option :- [:wat::WatAST])
  (:wat::core::if (:wat::core::>= i (:wat::core::length ch))
    (:wat::core::None :wat::WatAST)        ;; ⛔ type-checks; raises if this branch is taken
    …))
```

A **recorded, re-runnable migration** with a latent detonation in its `i >= length` arm. It has not
fired, which means that arm has not been taken on any run so far. **Left alone deliberately** — see
below.

## ⛔ THE RULING FAILED WITHIN ONE DAY — 2026-09-05

**A SECOND live site appeared, on this branch, in the same campaign, after this note was written.**

`wat-scripts/fanout/circuit.wat:121` held `(:wat::core::None :fanout::Seen::Reply)` in the drop arm of
`:fanout::seen`'s `claim` — the one arm arc 278's R2 stone exists to reach. It sat at HEAD, committed,
through a **green floor**, because rate 0 makes the arm unreachable: the same latency that keeps
`positional-to-kwargs.wat:27` quiet.

Measured A/B, one token, same fixture and seed:

| `circuit.wat:121` | `r2_drop_before_tiny` |
|---|---|
| `(:wat::core::None :fanout::Seen::Reply)` | ⛔ **TIMEOUT 30 s, empty arm** |
| `:wat::core::None` | ✅ **PASS 8 s**, `seen-dups=5` |

Its 30 s timeout was then written up as a **backpressure finding** and minted as a side quest (arc
278's S41, *"each dropped claim costs a 5000 ms deadline; 10 % of 8000 is saturation"*) — against a
deadline that had been 200 ms for two stones. The phantom form borrowed the meaning of its
surroundings a *second* time, in the *same* campaign, from the *same* author, and cost a spurious
stone in the order.

★ **This is the rung-1 argument, paid for twice.** *"On this branch, just avoid it"* is a convention,
and a convention is a failure class waiting for a tired afternoon. The gap that admits it is named
below: **Doctrine 1 catches type keywords in value position only for the primitives.** Extending that
rejection to every type keyword is rung 3 and would have made both sites unwritable.

⚠ **And the detection asymmetry is what makes it expensive:** the form is only reachable in an arm
the default configuration never takes, so a **green floor is not evidence of its absence.** Both live
sites were found by running the arm, never by a check.

## WHAT IT COST

An entire false investigation on `sns-sqs`, 2026-09-04, and it is worth recording because the
failure was not obvious from the inside.

A probe placed `(:wat::core::None :cd::Drop::Reply)` in a service arm's reply slot to test whether a
service could omit a reply. It type-checked, raised `UnknownFunction`, and **killed the service** —
and the caller's resulting `LOST` was read as *"the reply was omitted and the caller was told."*

From that misreading came, in order: a claim that a `None` reply is a userland fault-injection
mechanism; a stone (3d) drawn on it and struck down by the executor; a claim of *"two spellings of
one value with opposite runtime behaviour"*; and a proposed substrate stone against `Option`.

**None of it was real.** `Option` was never involved. The *"two spellings"* came from counting
`(:wat::core::None false)` **match arms** — pattern, then the arm's body, ~150 of them — as
constructor calls.

It ended on the builder's question: *"wtf is a typed none? how does this bear meaning? none… by
definition… holds nothing."* It does, and that was the whole answer.

★ **The lesson worth keeping is not about `None`.** A form that type-checks and then raises at
runtime produces a failure whose *symptom* is attributed to whatever it was near. Here the symptom
was a dead service in a fault-injection probe, so it was read as a property of fault injection.
**A phantom form does not announce itself; it borrows the meaning of its surroundings.**

## WHAT WAS AND WAS NOT CHECKED ON `main`

The builder believes this may already be addressed on `main`. **Not verified.** What was looked at,
read-only, no merge work:

- `main`'s `check.rs` carries the same `k == ":None" || k == ":wat::core::None"` keyword arms, at
  four sites (`:1950`, `:6422`, `:6519`, `:7064`) against this branch's three — so `main` has at
  least one additional arm (`match expected_ty`) not present here.
- Neither branch has a grep-visible guard phrased as *"not a function" / "takes no arguments"*.
- **Nothing was executed on `main`.** Whether that extra arm rejects the call form is unestablished.

⚠ `main` is mid-migration and **249 behind / 116 ahead**; the standing instruction is **no merge
work**. Confirming this on `main` means running the four forms in the table there, not reading a diff.

## THE RULING — on `sns-sqs`, avoid it; do not fix it

Builder, 2026-09-04: *"on this branch… just avoid the flaw."*

- **Write `:wat::core::None` bare.** It is the only correct spelling, and the `T` comes from context.
- **Do not add a checker guard on this branch.** If `main` has addressed it, a second fix is a merge
  conflict in `check.rs` during a migration that is already 249 commits behind.
- **`positional-to-kwargs.wat:27` is left as-is.** It is a recorded migration; editing a recorded
  artifact to repair a latent bug is a decision with its own consequences, and this note exists so
  the next reader finds it deliberately rather than by detonation.

`wat-scripts/scratch-pad/probe-none-is-not-a-function.wat` reproduces all four rows on demand.
