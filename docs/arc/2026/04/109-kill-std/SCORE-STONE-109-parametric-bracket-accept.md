# SCORE — 109 step ① (bracket accept)

Rider: two flights (initial + a resume for the room I missed), ~11 min total. Every row below re-run
by the orchestrator's own hand against the built binary.

| # | what | result |
|---|---|---|
| 1 | type position accepts the bracket | ✅ |
| 2 | nesting — the builder's pathological nest | ✅ `(Tuple [i64 String (HashMap [Keyword (HashSet [f64])])])` checks |
| 3 | `(Vector [i64] 1 2 2)` | ✅ **`[1 2 2]`** |
| 4 | `(Vector [i64])` empty instance | ✅ **`[]`** |
| 5 | `(HashMap [String i64] "a" 1)` | ✅ **`{"a" 1}`** |
| 6 | ★ **angle form still checks** | ✅ |
| 7 | ★ **`[A :-> B]` still checks** | ✅ |
| 8 | ★ positional form still BUILDS | ✅ `(Vector :i64 1 2 3)` → `[1 2 3]`, `(HashMap :String :i64 "a" 1)` → `{"a" 1}` |
| 9 | lexer untouched | ✅ no diff |
| 10 | renderer untouched (③'s job) | ✅ |
| 11 | `.wat` / `tests/` untouched by the rider | ✅ |
| 12 | `collection/eval.rs` net-zero | ✅ 0 lines |
| 13 | floor | ✅ **4818/4818, 69.8s** |
| 14 | clippy | ✅ 0 |
| 15 | rustfmt | ✅ HEAD=now on all three files — zero drift added |
| 16 | goldens | ✅ 7 bumped, two groups, see below |

Rows 6–8 were the load-bearing ones and all three hold: this step ADDED a spelling and removed none.

## ⛔ MY BRIEF WAS WRONG TWICE. THE RIDER CAUGHT ONE; THE SCORECARD CAUGHT THE OTHER.

**1 — Room 2's premise was false for half the arms.** I wrote *"each delegating to an `infer_*` that
reads leading type keywords off `args[0..]`."* Verified by my own hand:

```
infer_list / hashmap / hashset          WatAST::Keyword ×1, parse_type_expr ×1
infer_tuple / persistentmap / pvector   ZERO of each — every arg through infer()
```

The three that lack a leading-type path cannot take a spliced type keyword: `infer()`'s Doctrine-1
guard (arc 242, `check.rs:1894`) rejects a bare scalar type keyword in value position. The rider
STOP-3'd all three and documented each in place. **Correct call, and it found a general mechanism
where my brief had named one instance.**

**2 — THERE WAS A THIRD ROOM AND I MISSED IT ENTIRELY.** I mapped `check.rs` and assumed the
constructor lived only there. It does not: `runtime.rs` has its own constructor arms. After flight
one the checker ACCEPTED the bracket and the runtime answered *"malformed :wat::core::Vector form:
first argument must be a type keyword"* — a check-says-yes / runtime-says-no divergence.

★ **The only reason it surfaced is that EXPECTATIONS row 3 demanded a BUILT VALUE, not a green
check.** A scorecard written before the strike caught what the brief written before the strike
missed. That is the whole argument for writing them separately.

Second brief-authoring defect of the day, same class as the fs stone's extrapolated line numbers:
**rooms mapped from a partial read.** `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## ⚠ THE RIDER UNDER-CALLED ONE FINDING, AND IT GATES STEP ②

It reported `is_holon_arg_canonical` (`runtime.rs:29690`) as *"a missed fast-path… not a correctness
divergence."* The predicate's own doc says otherwise:

> *"This is what lets `(Bind (Atom "k") (Atom "v"))` fire as a single step instead of trying to step
> `(Atom "k")` separately and lift the typed leaf back through a primitive WatAST (**where it'd lose
> its HolonAST identity**)."*

The fallback is not merely slower — the doc says the value loses HolonAST identity on it. The arm
requires `items[1]` to be a bare `WatAST::Keyword`, so a bracketed `(Vector [T] …)` inside a
holon-constructor argument returns `false`.

**Unreachable today** (nothing writes the bracket yet) and **reachable the moment step ② lands**,
because after the codemod every `Vector` in a holon-constructor argument takes that path.

⛔ **STEP ② MUST NOT SHIP UNTIL THIS ARM ACCEPTS THE BRACKET.** Recorded here rather than fixed —
it is outside ①'s blast radius, and I have NOT proven identity is actually lost, only that the
predicate's own comment says the fallback is where that happens. Measure it before ②.

## The goldens — TWO groups, and the second nearly cost me

Seven red, all pinned lines. Hunks verified to precede every pin before applying any delta:

```
runtime.rs   hunks @6207 @6218 @6417   vs pin 25190   net +26 → 25216   (5 fixtures)
check.rs     hunks @56 … @11910        vs pins 13412/13430   net +76   (2 fixtures)
```

⚠ **The two `check.rs` fixtures pin DIFFERENT lines — 13412 and 13430.** My first `sed` matched only
13412 and left the other red. Same family as "the delta is not always uniform", one step further out:
*the pins are not all the same pin.* Both took +76 → 13488 and 13506.

## Honest deltas

- **All six `runtime.rs` line numbers in my resume message were correct** — the rider confirmed each
  by matching surrounding code rather than trusting the number, which is now the standing discipline.
- **The rider chose reuse over duplication** and argued it: `unwrap_type_param_bracket` is pure
  `&[WatAST] → Cow<[WatAST]>` with no check-specific state, so it made it `pub(crate)` and called it
  from `runtime.rs` rather than writing a twin in `collection/eval.rs` — citing the existing
  `collection → check` and `runtime → check` dependency directions. `collection/eval.rs` ends net-zero.
- **It spliced at the dispatch call site, not inside the `eval_*_ctor` bodies**, mirroring Room 2's
  own pattern. The callee fns are untouched in both rooms.
- **The builder's illustrative `HashMap [Keyword String]` example does not build — and it is NOT this
  stone.** `expects :wat::core::Keyword; got :wat::core::keyword`. The POSITIONAL form fails
  identically, which is the control that proves it pre-existing. Capital-`Keyword` has **0 references
  in `wat/`** against lowercase's **276** — a type the checker knows and the corpus never uses. With
  lowercase it builds verbatim: `{:some-kw "some-str"}`. Filed as 109-shaped (PascalCase), not ①'s.

## What ① did NOT do — so nobody reads it as complete

Three constructors accept the bracket in BOTH rooms: `Vector`, `HashMap`, `HashSet`. Three do not, in
either: `Tuple`, `PersistentMap`, `PersistentVector` — consistently STOP-3'd on both sides, because
three correct in both rooms beats six half-done in one. **Extending to those three requires modifying
their `infer_*` fns**, which this brief forbade and which is its own strike.
