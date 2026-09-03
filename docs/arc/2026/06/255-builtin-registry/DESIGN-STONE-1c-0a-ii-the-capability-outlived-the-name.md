# DESIGN — STONE 1c-0a-ii: three repoints. **The capability outlived the name, every time.**

> **Builder's ruling, 2026-09-03:** *"deletions must clear a high bar... these do not meet the
> requirement for deletion.. we augment as they need."*
>
> Supersedes the disposition half of
> `[[DESIGN-STONE-1c-0a-five-call-sites-name-nothing]]`. Its PART 1 (two namespace slips, by
> codemod) landed at `63e7a0c7f` and stands. **Its reasoning about the other three does not.**

## ⛔ WHAT I GOT WRONG, THREE TIMES, IN ONE SHAPE

I recommended deleting two artifacts and declared a third capability absent. All three were the
same error:

```
searched for            found                     concluded              THE TRUTH
:wat::core::reduce-walk    deleted 2026-08-18     ARM B unrepairable     :wat::core::foldl-spec-walk
                                                                          — SAME signature, live
:wat::spawn::process/grants deleted 2026-07-08    probe unfixable        (:wat::spawn::process)
                                                                          — gives the probe's exact
                                                                            stated condition
tuple-* / Tuple/* names     none exist            "no Tuple accessor"    :wat::core::first
                                                                          — polymorphic over Tuple
```

★★★ **I searched for the retired NAME, found it gone, and concluded the CAPABILITY was gone.**
The capability outlived the name every single time. Each conclusion then went into a
four-questions table that read as rigorous — because the tables were sound *given the options I
supplied*. **The four questions cannot see a missing option; the failure was upstream of them
every time,** and the correction came from the builder asking one question I had not.

## The three repoints, and why each is faithful rather than a weakening

**① `bench-reduce-foldl-vs-seqable-walk.wat:30` — `reduce-walk` → `foldl-spec-walk`**

```
ARM B as written   (:wat::core::reduce-walk       f 0 (:wat::core::Seqable/seq v))
what exists        (:wat::core::foldl-spec-walk  [f <- [U T :-> U]  acc <- :U
                                                  s <- (:wat::stream::Stream :- [T])] -> :U)
```
Same arity, same order, same types. And not a coincidence: `wat/seq.wat:265` attributes the
bench's `5.1×` **to `foldl-spec`'s slowness**, whose walker IS `foldl-spec-walk` — a verb that is
live and deliberately kept (*"the native kernel is the fast impl, the spec keeps it honest… its
slowness is the design, not a defect"*). The repaired bench measures exactly what its three
citations already say it measures.

**② `probe-cap2-process-grantpath.wat:10` — drop the retired combinator**

```
(:wat::bracket::map (:wat::spawn::process/grants
                      (:wat::core::Vector :- [:wat::capability::Grantable])) nums :probe::double)
→  (:wat::bracket::map (:wat::spawn::process) nums :probe::double)
```
The probe's own header wants *"grantables is EMPTY, so the foldl is a no-op — but the peer-pid
Some branch + the whole map-worker grant/revoke path EXECUTES."* `wat/bracket.wat:734` says a
plain pool passes `grant-handles = nil` with *"a no-op grant-fn… calling it unconditionally is
harmless"*, and the `peer-pid → Some pid` match at `:738` still fires. **Plain `(process)`
produces precisely the condition the probe describes.**

⚠ And it carries a SECOND dead name: `:wat::capability::Grantable` was renamed to
`:wat::capability::Capability` (stone A); only prose mentions survive. **That name never appeared
in the 71-name corpus census at all** — the census collects call HEADS, and this one hides in TYPE
position. The blanket accept launders more than the experiment can see.

**③ `arc109-2iii-fn-bracket-destinations.wat:55` — `tuple-get` → `first`**

`check.rs`'s `infer_positional_accessor`: *"Polymorphic over (Vector :- [T]) and tuple — both are
index-addressed."* `wat/rete.wat:300` reads a live `(Tuple :- [Record i64])` with `first`.

## ★ WHY NONE OF THESE IS A DELETION CANDIDATE — the fixture argument

`every_wat_scripts_file_loads` parses and type-checks **557** `.wat` files, **83** of them under
`wat-scripts/probes/`. These artifacts are checker integration fixtures, passively, at scale.
A fixture's value is that **it type-checks a real path** — not that something imports it. My
earlier *"zero references, therefore a clean deletion"* was true and entirely beside the point.

⚠ And the grantpath probe is not redundant with `probe-cap2-isolate.wat`: that sibling carries a
full `defsurface` + `defservice`; this one is deliberately minimal — *"WITHOUT any user record
(dodging the pre-existing freeze bug where `type_def_to_ast` drops a user record's fields)"*. That
minimality IS its fixture value.

## THE FOUR QUESTIONS — the disposition, re-posed with the option I had missed

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **repoint all three to their live spellings, and augment each header** | YES | YES | YES | YES | ✅ |
| delete the two unrunnable artifacts | YES | YES | **NO** | — | ⛔ |
| keep them as-is | **NO** | YES | **NO** | — | ⛔ |

**Delete is dishonest** on the ruling's own bar: each is repairable in one or two tokens, each
becomes a live fixture again, and one of them re-arms a measurement three documents cite.

## Acceptance — DERIVED

```
                        before   after   why
the corpus 68             68      65     −3 names: reduce-walk · process/grants · tuple-get.
                                         ⚠ :wat::capability::Grantable also leaves the tree but
                                         was NEVER in the 68 (type position, not a call head).
files deleted              —       0     the ruling's bar
files repaired             —       3     each runs or type-checks a real path again
GAP_A / GAP_B / DEBT   60/68/106  same   nothing is registered by this stone
floor              5127/5127  5127/5127
```

★ **The bench becomes runnable again, so this stone can re-derive a number that three documents
currently carry on 2026-08-18's authority alone.** The rider RUNS it and REPORTS the figure; it
does not touch the citations — a materially different number is a finding for the builder, not an
edit.
