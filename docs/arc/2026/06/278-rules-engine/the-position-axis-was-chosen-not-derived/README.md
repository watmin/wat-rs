# the position axis was chosen, not derived — ⛔ TITLE STRUCK, see the FINDING's amendment

**Opened 2026-09-10**, out of the chain that began with the `where` fence's type hole.

The rete validator is checked along two axes: **which OP** (the `RETE_OPS` row) and **which
POSITION** (where in a rule the expression sits). One of those axes is enumerated from a type and
is exhaustive. The other is a hand-written list with **two** members, while the language declares
**eleven** clause shapes.

That asymmetry is why this arc keeps finding the same defect one instance at a time — D10 (`:then`
RHS, cured 2026-09-02), D11 (nested `:then`), the `where` fence (cured 2026-09-10), and now
`accumulate`'s `:from`.

| file | what it holds |
|---|---|
| `FINDING-the-position-axis-was-chosen-not-derived.md` | the driven coverage table across eight positions, and the two-member enum |

## The state, in one line

**Seven of eight positions correctly refuse an ill-typed predicate. `accumulate`'s `:from` does
not.** The instrument that should have told us this asks about two of the eight.
