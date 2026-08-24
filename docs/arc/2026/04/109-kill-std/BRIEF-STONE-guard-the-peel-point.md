# BRIEF — STONE: guard the peel point

Too many call-site type arguments are silently swallowed, for every callee in the language. Add one
condition at the single place type arguments are consumed.

Design: `DESIGN-STONE-guard-the-peel-point.md` — **read it first**; it carries the measured
reproduction, the history (this class has bitten twice before), and what is affirmatively cut.

## Reproduce it first — fix the thing you measured

```
(:wat::eval-ast! :- [:wat::core::i64 :wat::core::String :wat::core::bool] <ast>)   → Ok [42]
(:wat::eval-edn! :- [:wat::core::i64 :wat::core::String :wat::core::bool] "42")    → Ok [42]
```

`eval-ast!` declares ONE type param; `eval-edn!` declares ZERO. Both accept three and drop the extras
without a word.

## Read in order — verified this session

1. **`src/check.rs:5635-5638`** — the ONLY consumer of `type_args`:
   ```rust
   let (param_types, ret_type) = match &type_args {
       Some(concrete) => instantiate_with_args(scheme, concrete, fresh),
       None            => instantiate(scheme, fresh),
   };
   ```
   **The guard goes here**, before the call.
2. **`src/check.rs:16194`** — `instantiate_with_args`. Read it to see WHY extras are unreachable: it
   iterates `scheme.type_params` and indexes into `type_args`, and early-returns when the scheme has
   no params. **Do not change this function.** It is correct for what it does; it is simply never
   told that the input was wrong.
3. **`src/check.rs:4977`** — the one peel point, ~660 lines above. Its comment records the two prior
   instances of this class. Read it for the shape of the sibling diagnostic at `:4986`
   (`MalformedForm`, "malformed type-param argument") — **match that family.**

## The rule — write it as `>`, not `!=`

```
concrete.len() > scheme.type_params.len()   →   REFUSE
```

`>` is load-bearing and is the whole design:

- **FEWER than declared stays legal** — inference legitimately completes a partial application.
  `!=` would break every such call.
- **`:- []` stays legal everywhere** — `0 > N` is false for all N, so the empty binder is admitted
  **by construction**, not by a special case. `:- []` ≡ absent is arc 109's ruling and macros emit it
  unconditionally; a guard that needed an exception for it would be the wrong guard.

## The diagnostic

`CheckErrorKind::MalformedForm`, matching the sibling at `check.rs:4986`. **No new variant.** The
`reason` must name the callee AND both counts — a reader must learn what they wrote and what was
expected without opening the source.

## Blast radius

`src/check.rs` — the guard at `:5635` only. Plus your probe and fixture. **No change to
`instantiate_with_args`, to the peel point at `:4977`, to `check/error.rs`, or to any runtime file.**

## The probe you create

`tests/types/probe_stone_guard_the_peel_point.rs` + co-located `.wat`. Five rows:

1. **one param, three args** — `(:wat::eval-ast! :- [A B C] e)` REFUSED, diagnostic names the callee
   and both counts. *Load-bearing.*
2. **zero params, one arg** — `(:wat::eval-edn! :- [A] "42")` REFUSED.
3. **empty binder** — `(:wat::eval-edn! :- [] "42")` **ACCEPTED**, identical to no binder.
4. **exact count** — a genuine generic call with exactly its declared count still works.
5. **FEWER than declared** — a generic call supplying fewer type args than params still infers and
   works. **This is the row that proves the guard is `>` and not `!=`.** Do not drop it.

Rows 3 and 5 are the ones that fail if the guard is written wrong in the tempting way.

## STOP triggers — rejection criteria. Ship nothing on the row; report it.

1. **`scheme.type_params` is not reachable at `:5635`.** Report the exact compiler error. Do NOT move
   the guard inside `instantiate_with_args` as a workaround — that function has other callers
   (`instantiate`), and the design pins the guard at the consumer.
2. **Row 5 fails** — some existing call supplies fewer type args than declared and now breaks. That
   means the guard is wrong or partial application is used more widely than assumed. STOP and report
   the failing call sites verbatim.
3. **The floor goes red beyond your probe.** Any existing call in the corpus supplying MORE type args
   than declared is a REAL defect this guard just surfaced — capture every one verbatim and STOP.
   **Do not "fix" corpus call sites to make the floor green**; that is the finding, and it is the
   orchestrator's to rule on.

STOP-3 is the likely one. Treat a cascade as the substrate teaching, not as your stone failing.

## Method

`cargo nextest run --release -E 'test(probe_stone_guard)'` and your `.wat` through
`target/release/wat`. Report those numbers. Run everything in the FOREGROUND and block on it. Do NOT
run the full floor or clippy — the orchestrator runs those centrally.

Do not commit, push, stash, or amend. Leave the git index empty. You may not spawn sub-agents.

## Report

Five rows with actual results; `git diff --stat`; the exact diagnostic text row 1 produces; and if
STOP-3 fired, every offending call site verbatim.
