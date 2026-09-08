# DESIGN — STONE N: the bare variant spelling is HERESY, and the screams are the census

> **Builder, 2026-09-07:** *"annihilation is our greatest pleasure - the illegal forms must go… make
> the illegal forms actual heresy - the heretics self identify as we set them ablaze - the
> shadowdancer seeks them out by their screams"*

## ⛔ THIS SUPERSEDES THE M2 RELAND. THE FOUR QUESTIONS REVERSED THE ORCHESTRATOR'S CALL.

M2 left three residues, and the orchestrator proposed a narrow reland: unwrap 118 damaged sites,
extend a gensym table, fix one file. **Run flat, the four questions refused it.**

```
                                    Obvious  Simple  Honest  Good UX
(a) narrow reland, four bare
    names stay positional             YES     YES     YES     ⛔ NO   two conventions live at once;
                                                                     a caller must count FQDN
                                                                     segments to know which applies
(b) retire the bare spelling
    FIRST, then wrap uniformly        YES     YES     YES      YES    one rule, no exception
```

The orchestrator had argued (a) on **risk** — an axis the four questions do not measure — and the
risk claim did not survive either: it counted STONES, not HAZARDS. A rename cannot produce
`Option<HashMap<…>>`; a conditional wrap already did, 118 times.
`[[feedback_an_unmeasured_axis_cannot_break_a_tie]]`

## ★★★ AND THE RENAME REPAIRS THE DAMAGE. MEASURED, NOT ARGUED.

M2's codemod wrapped 118 bare heads — `(:wat::core::Some x)` → `(:wat::core::Some {:value x})` —
which types as `Option<HashMap<keyword,T>>`, the map swallowed as payload. Fixture-local errors:

```
(:wat::core::Some         {:value 1})     1      ← the regression, 118 of these on disk
(:wat::core::Option::Some {:value 1})     0      ← after the rename
```

**No unwrap pass is needed.** Those 118 sites become CORRECT the instant their heads are qualified,
because M intercepts three-segment variant FQDNs and `:wat::core::Some` is two. grok's
`unwrap-alias-edits` is unnecessary and must not be run.

## THE METHOD IS THE BUILDER'S: IMPOSE THE WALL, READ THE SCREAMS

**The wall is the census instrument, not just the end state.** Every count in this document is a
GREP and is therefore an estimate — the same instrument that has been wrong nine times in this arc.
The wall's error stream is a FORM-TREE census: it cannot miss a macro-generated site, a spelling
nobody thought to pattern, or a construction that only exists after expansion.

```
grep says (2026-09-07, post-M2-partial):
  :wat::core::Some 655 · None 669 · Ok 366 · Err 349  = 2,039
  the arc-109 legacy bare `:None`                     =    31
⛔ THESE ARE ESTIMATES. THE SCREAMS ARE THE WORKLIST.
```

`[[feedback_impose_the_check_and_read_the_screams]]` · 296 R20 `HAERESIS EST ITERVM ROGARE`

## WHERE THE HERESY IS ADMITTED TODAY — the arms that must scream

```
src/match_arm.rs:161-164   the four `builtin_variant` arms; each accepts BARE | QUALIFIED
src/runtime.rs:1700 · 8668  `:None` || `:wat::core::None` || `:wat::core::Option::None`
src/runtime.rs:8761 · 8792 · 8820   Some / Ok / Err, bare || qualified
src/runtime.rs:13427-13429  a second Some/Ok/Err set
src/declare/register.rs:1155 · 1167   registration
                            62 Rust sites total across 10 files
```

★ The refusal must **NAME ITS REPLACEMENT**, as M's and `assertion-failed!`'s do — *"the bare variant
spelling is retired; write `:wat::core::Option::Some`"*. A silent fall-through is not a scream; it
is the thing that let `:wat::core::None` survive four arcs.

## WHAT IT DELIVERS

```
(:wat::core::Some x)        ->  (:wat::core::Option::Some {:value x})     via N then M2's wrap
:wat::core::None            ->  (:wat::core::Option::None {})
(:wat::core::Ok  x)         ->  (:wat::core::Result::Ok  {:value x})
(:wat::core::Err e)         ->  (:wat::core::Result::Err {:error e})
[:wat::core::Some {:value v} …]  ->  [:wat::core::Option::Some {:value v} …]   patterns too
```

One spelling, one construction grammar, no segment-count exception. And the tree RUNS again — which
is the only state that matters.

## ⛔ SCOPE — WHAT IS IN, AND THE ONE THING THAT IS NOT

**IN:** the five illegal spellings (`:None` and the four bare `:wat::core::` heads), the corpus
rename, M2's remaining wrap work (the `service.wat` template heads), and the Rust arms that admit
the heresy — **because a rename that leaves the door open is not an annihilation.** The bare
spelling must become UNREPRESENTABLE, not merely unused.

**OUT:** `{:keys}` on `defrecord` · `variant <: enum` · the `:wat::*` whitelist. Builder-ruled as
their own stones.

## THE FOUR QUESTIONS ON THIS STONE

- **Obvious?** YES. One spelling for one thing, and the refusal names its replacement.
- **Simple?** YES. A wall, a rename, and the wrap that already exists — each single-purpose.
- **Honest?** YES. It annihilates the form rather than deprecating it; the door closes in the same
  stone that empties the room.
- **Good UX?** YES. No exception for a caller to learn, and every stale site says exactly what to
  write instead.
