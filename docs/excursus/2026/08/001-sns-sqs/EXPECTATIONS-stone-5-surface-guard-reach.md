# EXPECTATIONS — excursus 001 stone 5: the guard's reach

**Written BEFORE the strike, 2026-08-31.** Blast radius derived from the BRIEF's own section.

## ⚠ THIS STONE IS EXPECTED TO TURN THE FLOOR RED

`wat-scripts/queue/sqs.wat` carries the exact shape the guard currently misses. **After the fix
it must stop freezing** — that is the guard working, not a regression. The floor will be red on
the queue until stone 6 moves `Envelope` into `:messages`.

**Expected: RED, and only on the queue.** Red anywhere else is this stone's.

## The scorecard

| # | what | expected |
|---|---|---|
| 1 | ★ the parametric repro now fails | `repro/parametric-field-type.wat` `--check` → **1** (it is `0` today) |
| 2 | ★ the direct repro STILL fails | `repro/direct-field-type.wat` `--check` → **1**, unchanged — a widening, not a rewrite |
| 3 | the message is the existing one | names `:p::Item`, byte-identical, no `.contains(` |
| 4 | a clean surface still freezes | the guard did not start rejecting valid code |
| 5 | `collect_user_type_paths` untouched | `git diff` shows no change to `:974-995` (STOP-1) |
| 6 | the queue now fails to freeze | **expected and correct** — say so in the SCORE, do not fix it |
| 7 | nothing ELSE started failing to freeze | any other surface that breaks is a **finding** (STOP-2) |
| 8 | blast radius | `src/types/surface.rs` + its test + SCORE |
| 9 | floor | RED on the queue only |

## Runtime prediction

**45–90 minutes.** One branch plus a test. The unknown is rendering a `WatAST::List` type form
to a `TypeExpr` — the codebase does this elsewhere and the door should be reused, not rewritten.

## Trap-doors

1. **★ Row 7 is the interesting one.** The guard has been blind to parametric field types since
   arc 278. **Any surface in the tree could be carrying the same latent defect**, and widening
   the reach is what surfaces them. Each one that appears is a real fork-failure waiting to
   happen — name them, do not fix them, and do not let their count discourage the widening.
2. **Row 2 can regress silently.** If the fix restructures the `<-` handler rather than adding a
   branch, the keyword path can break while the parametric path starts working, and row 1 would
   go green while row 2 quietly stops testing anything.
3. **A type var is not a user type.** The existing guard exempts `:wat::` types and type
   variables. A parametric like `(Vector :- [T])` must stay exempt on `T`; only namespaced user
   types are protocol messages.
4. **Nested twice.** `(Vector :- [(Option :- [:p::Item])])` must also be caught —
   `collect_user_type_paths` recurses, so this should come free. Worth one assertion to prove
   the reach is real and not one level deep.

## Not in this stone

- **Moving `Envelope` into `wat-queue`'s `:messages`** — stone 6, and the floor is red between.
- **`UnresolvedReference`'s `&'static str` context** — a real separate weakness, recorded in
  the SCORE of stone 4, not drawn.
- **Any other surface this widening exposes** — findings, not fixes.
