# BRIEF — STONE N RELAND 8: ⛔ THE RENAME CORRUPTED `Option`'s OWN DECLARATION

> **This is upstream of RELAND 7 and of several "fixes" layered above it. Guard the tool, repair
> the declaration, revert the workaround, re-measure. In that order — the builder has ruled it.**

## THE DEFECT — reflection's own words

`wat/core.wat:2127`. Before this campaign, and now:

```wat
BEFORE (46b7f9c78)                      NOW
  (defenum :wat::core::Option …           (defenum :wat::core::Option …
    :Some [value <- :T]                     :Some [value <- :T]
    :None)                                  :wat::core::Option::None)      ⛔
```

`:wat::runtime::type-of :wat::core::Option` — the instrument, not a grep:

```
:variants [ { :name :Some                 :fields [ {:name :value :type T} ] }
            { :name :wat.core.Option/None :fields [] } ]        ⛔
```

**`Option`'s unit variant is DECLARED under the name `wat.core.Option/None`.** Its resolved path is
therefore `Option::wat::core::Option::None` — a name nothing resolves. That is why the purity
lattice reports *"`:wat::core::Option::None` is not proven pure"* and why `unit_variants` lookups
miss it.

`Result` escaped **only** because `:Ok`/`:Err` carry field vectors the rename's pattern did not
match. `:None` was a bare unit variant — textually indistinguishable from a *use* of `:None`.

⛔ **`wat/core.wat` is `include_str`'d into the binary.** A re-corruption is not one bad file; it is
every build shipping an `Option` whose `None` is unnameable — silently, past the type checker.

## THE ORDER (four-questions ruled; UX broke the tie)

**1. GUARD THE CODEMOD FIRST.** A `defenum` has known structure — head, type name, `:- [params]`,
purity marker, then *variant-name / field-vector* pairs. **A variant-name position is not a use
site.** Teach `positional-ctor-to-map.wat` AND the rename pass to skip it. This lands before the
repair so the repair cannot be silently undone by the next sweep.

**2. REPAIR THE DECLARATION — to `:None []`, not to `:None`.** The builder's ruling: a unit variant
declares an explicit empty field vector, uniform with payload variants and with the forms that
consume it.

```
declare   :None []          construct   (…::None {})        match   [… ::None {} body]
```

Measured: `:Empty []` already checks clean (`check=0`). Reflection already reports `:fields []`. No
grammar work — one token.

**3. REVERT THE `wat/query/mem.wat` KEYFN WORKAROUND.** RELAND 7 rewrote
`(sort-by :wat::query::Row/sk matches)` to an explicit `fn` because `classify_closure` refused the
accessor. ★ **That refusal is very likely a CONSEQUENCE of the corruption, not a classifier gap.**
Arc 255 Stone A-2-ii-b-0 ruled those accessor verbs Pure ∧ Deterministic **for this exact call
site** — see `wat-scripts/scratch-pad/255-probe-the-accessor-classifies-pure.wat`, whose header
names `mem.wat:136,163` verbatim. Restore the keyword keyfn and MEASURE. If it now classifies pure,
the workaround was compensating and the property arc 255 established is restored. If it still
refuses, THAT is the classifier gap — and it is then a real, separately-named finding.

**4. RE-MEASURE THE 34.** RELAND 7's `-11` cluster, the rete smem/sqlite set, and whatever else the
repair heals. Report per cluster.

## STOP TRIGGERS

- **STOP-1 — the declaration is repaired before the codemod is guarded.** The builder ruled the
  order; an unguarded tool re-poisons the frozen stdlib.
- **STOP-2 — the keyfn workaround is kept without measuring.** It may be compensating for the
  corruption. Restore, measure, then decide — and say which.
- **STOP-3 — the guard is a name blacklist.** It is a POSITION rule: inside a `defenum`, the
  variant-name slot is not a use site. A blacklist of `:None`/`:Some` is the hand-list this arc
  keeps deleting.
- **STOP-4 — other declarations are assumed clean.** Ask `type-of` for every enum the campaign
  touched; a corrupted variant name type-checks and says nothing.
- **STOP-5 — a test expectation is updated to match a wrong value.** Still standing from RELAND 7.
