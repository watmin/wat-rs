# EXPECTATIONS — the lattice keys on the base name

Written BEFORE the strike. Every row independently re-run by the orchestrator.

| # | what | expected |
|---|---|---|
| 1 | edge from `(extend-type :Vector :Seqable<T>)` found by a query for `:Seqable` | found |
| 2 | …found by a query for `:Seqable<T>` and for `:Seqable<?N>` | found — **this is the defect closing** |
| 3 | the same edge written as a FORM parent `(:Seqable :- [T])` | found |
| 4 | `transport_satisfier_heads` returns ONE key | no `format!("{fq}<T>")` / `<Xt>` remains |
| 5 | `satisfies_bare_surface`'s `format!("{surface}<")` prefix match | GONE |
| 6 | `grep -rn 'format!("{fq}<\|format!("{surface}<' src/` | returns nothing |
| 7 | ★ arc 293 transport — `Handle<Wire>` satisfies bare `Dialable` | still passes |
| 8 | `is_subtype`'s 30 call sites | UNCHANGED — signature does not move |
| 9 | stones 2 and 3 untouched | `git diff --stat` shows no `service.wat` / `core.wat` / `bracket.wat` / `fix.wat` |
| 10 | floor (orchestrator, central) | **4854/4854** — or a NAMED set of transport reds |
| 11 | clippy `-D warnings` | 0 |

## Independent prediction

**20-40 min.** Three touch points in one file plus two deletions with six call sites. **2× box: 80 min.**

## Trap doors, named before the strike

1. ★ **Row 2 IS the stone.** A query by the exact registered spelling passes today; a query by a
   *different* spelling of the same type is what fails. A rider that only tests row 1 will see green
   and have changed nothing observable. **One spelling proves nothing** — the same shape that made
   γ-i's row 7 vacuous, hours ago.
2. ★ **STOP-1 is an OUTCOME.** Arc 293's `transport_satisfier_heads` guesses `<T>` and `<Xt>` on the
   SUB side too. If stripping breaks it, some edge genuinely needed an instantiation and the design
   gains a named exception. A rider that special-cases past it hides the finding.
3. **Row 9 is the B-3 boundary made mechanical.** This is stone 1 of 3; a diff reaching `defservice`
   means the split ruled an hour ago is already leaking.
4. **Base extraction must be SINGULAR inside the lattice.** Sixteen hand-rolls exist elsewhere; a
   seventeenth added here would reintroduce exactly the inconsistency being removed — even while
   every row passed.
5. **A green floor is not sufficient.** Rows 2 and 3 must be seen going from FAILING to PASSING.
   A stone whose only evidence is "nothing broke" has not shown that anything now works.
   `[[feedback_a_green_test_can_prove_nothing]]`

## Scoring method

Written after the orchestrator's own re-run. Row 2 is checked FIRST, against the pre-strike build as
well, so "it always worked" and "the stone fixed it" cannot be confused.
