# NOTE — defservice clauses should adopt the arc-293 Holder vocabulary (a deferred rename)

**Status: QUEUED (builder-approved 2026-06-26). Do AFTER the 293 `:holder` additive layer lands — do NOT braid
into it.**

`defservice`'s state clauses predate arc 293's HOLDER × SURFACE model and were named before the vocabulary was
won. Arc 293 R3 records the alignment outright: defservice's `:ephemeral` / `:durable` / `:durable-parent :holon`
**IS the Holder ladder before the word existed** —

| defservice clause (today) | Holder | the 293 vocabulary |
|---|---|---|
| `:ephemeral` | `Struct` (−1, in-locus, never crosses) | (reconsider against `Struct`) |
| `:durable` | `Record` (0, EDN, crosses) | (reconsider against `Record`) |
| `:durable-parent :holon` | `HolonRecord` (+1, EDN + VSA) | `:holder :holon-record` |

`defsurface` and `:holder` came out of a long naming debate (intueri) and the builder prefers them strongly over
the defservice-isms. The move: **teach defservice the won Holder vocabulary** — at minimum `:durable-parent :holon`
→ the `:holder :holon-record` spelling (which maps 1:1 onto `Holder{Struct,Record,HolonRecord}`, cleaner than the
`:holon` defservice-ism); reconsider `:ephemeral`/`:durable` against `Struct`/`Record` in the same cast.

**Why deferred / separate strike:** its own blast radius — `wat/service.wat` (the clause parser + the
`:durable-parent` handling at `service.wat:169-283`) + every `defservice` call site + a retirement-table entry so
the old clause spellings throw a teaching error, not silently drift. This is an **intueri cast** in its own right
(propagating the won naming quality), not a mechanical find-replace.

Pairs: `293-struct-record-symmetry/DESIGN.md` § THE HOLDER × SURFACE MODEL (the `:holder` clause + the `defservice`
precedent) · `293-…/REALIZATIONS.md` R3 (the trit / "the name was already yours") · `wat/service.wat:58,126,169,283`.
