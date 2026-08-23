# BRIEF — reap the angle machinery

`<K,V>` became unexpressible in `0811c3009`. The machinery that parsed it is still compiled and still
called **16.2 million times per floor run**, finding a type-head **zero** times. You will remove it.

Read `DESIGN-STONE-reap-the-angle-machinery.md` first — it carries the measurement. The tree is CLEAN
and the floor is green at 4903/4903. Copy the report shape of
`SCORE-STONE-the-last-comma-lives-in-a-symbol.md`.

## The measurement you are acting on

```
                                 calls        type-heads found    callers
canonical_callable_name       14,481,408              0              13
split_name_and_type_params     1,128,732              0              11
split_type_params                617,719              0               4
split_method_name_type_params          0        never called          4
```

Of ~25,400 calls whose argument contained an angle at all, **every one was the operator `::<`** —
which these functions deliberately do not strip.

## ⛔ READ THIS BEFORE THE FIRST DELETION

```clojure
:wat::core::<     :wat::core::>     :wat::core::>=     <-     ->
```

Operator names contain `<` and `>`. The balanced-suffix rule (*strip only when the name also ends in
`>`*) exists precisely to protect them, and the 25,400 measured calls ARE those operators flowing
through. **A purge that removes the rule along with the parser takes the operators with it.** Your
acceptance row 1 is that they still dispatch.

## STEP 1 — the one that is genuinely dead

`split_method_name_type_params` (`src/types/surface.rs`): **0 calls, 4 callers.** Callers that never
execute are a different finding from a no-op that always executes. **Say which of the four are
unreachable and why** — do not delete the function and let the compiler find them for you. If one of
the four IS reachable and the census simply never exercised it, that is STOP-1.

## STEP 2 — the three no-ops

`canonical_callable_name`, `split_name_and_type_params`, `split_type_params`. Their call sites should
use the name directly. Work outward from each call site rather than deleting the function first — the
compiler errors will find the sites, but the errors will not tell you whether a site *meant* something
by the strip.

⚠ For each call site ask: **did this site depend on the strip, or merely tolerate it?** A site that
looked up a stripped name and now looks up the raw name is only correct because no name carries a
suffix any more. Say so where it applies; that is the reasoning a future reader needs.

## STEP 3 — the two callers that are NOT purge candidates

`split_type_params_pub` has two live callers, neither parsing a call head:

```
src/types.rs:836          base_of_rendered_type     strips a base off a RENDERED type string
src/types/surface.rs:997  message_is_declared       same
```

The renderer now emits `(Head :- [args])`, so a rendered string contains **no `<`**.

⚠ **This is the shape that has bitten three times in this arc:** *a search for a character that no
longer exists does not fail — it succeeds wrongly.* `base_of_rendered_type` was already taught the
`:-` form during the renderer stone. **`message_is_declared` was never examined.** Examine it. If it
hunts for a `<` that can no longer appear, it is silently returning the whole string, and that is a
live bug — report it as a finding, not as cleanup.

## STEP 4 — the stale comment and the wall

`check.rs:4964` still describes the explicit-type-suffix arm, which the call-position hoist already
deleted in `c6c614fe2`. Prose describing a mechanism that no longer exists — remove or correct it.

Then extend `tests/lint/one_param_spec.rs` (or add a sibling rune) so a hand-rolled `<`-parse of a NAME
cannot return. `tests/lint/one_name_grammar.rs` is the shape and both are already positive-controlled.
**Positive-control yours the same way**: plant a violation, confirm it fails and names the site, remove
the plant. A rune that has never failed has not been shown to work.

## Acceptance

| # | what | expected |
|---|---|---|
| 1★★★ | the operators still dispatch | `(:wat::core::< 1 2)` → `true`; `>`, `>=`, `<-`, `->` all live |
| 2★★★ | a surface-method call still resolves | `(:S/method recv arg)` returns its value |
| 3★★ | a parametric `defservice` round-trips | lru-svc / hologram-svc |
| 4★★ | the four functions are gone | or, for `split_type_params_pub`, only its rendered-string use remains |
| 5★★ | `message_is_declared` | examined, and what you found |
| 6★ | the rune | drawn, and positive-controlled |

**Row 1 decides it.** Rows 2-4 go green for a purge that also removed the balanced-suffix protection —
the operators are what that rule existed for, and they are the 25,400 calls the census actually saw.

## STOP triggers

- **STOP-1 — a caller of `split_method_name_type_params` IS reachable.** The census measured zero
  calls; a reachable caller means the census missed a path. Report it; that is the finding.
- **STOP-2 — a call site DEPENDED on the strip** rather than tolerating it, so removing it changes
  behaviour. Report the site and both behaviours.
- **STOP-3 — `message_is_declared` is searching for a `<` that cannot appear.** Report it as a live
  bug with what it returns now; do not fold the fix silently into a deletion commit.

## Boundaries

- `src/runtime.rs`, `src/check.rs`, `src/types.rs`, `src/types/surface.rs`, and one rune.
- **Do NOT touch `keyword/to-type-form` / `to-type-form-colon`.** Their live caller at
  `wat/service.wat:434` is a transition shim; its contract is a separate question.
- **Do NOT touch `keyword/from-string`'s NAME.** Its own NOTE.
- **Do NOT sweep the retired spelling out of COMMENTS.** 411 wat + 591 rust lines, FM 14 Bucket B, its
  own stone — and a blind pass would erase the lines that RECORD the retirement.
- Do NOT commit, push, stash or amend. Keep the git index EMPTY: no `git add`, no
  `git checkout <ref> -- <path>` (it STAGES).
- Goldens: **KEEP PINNING THE SPAN** and recapture; verify each is the same call site, only moved.
- The orchestrator runs the full floor and clippy centrally.

Build with `systemd-run --user --scope -q -p MemoryMax=24G -p MemorySwapMax=0 timeout 3000 cargo build --release`.
Read exit codes DIRECTLY — never through a pipe, never after a trailing `; echo`.
`cargo wat` uses the STALE installed binary; always `./target/release/wat`.

## Your report

Row 1 first — the operators still dispatching — because that is the row a careless purge fails. Then
rows 2-6. Which of `split_method_name_type_params`'s four callers were unreachable and why. What you
found in `message_is_declared`. For each deleted call site, whether it DEPENDED on the strip or merely
tolerated it. Any STOP that fired, with the arm captured verbatim BEFORE you diagnosed it. What
surprised you.
