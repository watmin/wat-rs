# 296 · DESIGN STONE J — every type wat uses is declared in wat

> **The class, not the instances.** Builder, 2026-09-06: *"are there enums that wat uses that are
> defined by hand in rust code, not wat code? … those 25 … they are trivial to move and trivial to
> source? … we purge the bad behavior for all time?"* and *"we should use as much wat as we can…
> dog food our tooling maximally. not wasteful, unnecessary stuff but honest use of tooling to
> prove we are continually functional."*

## ⛔ THE ORCHESTRATOR'S BENCHMARK WAS WRONG, TWICE, AND THAT IS WHY THIS STONE EXISTS

I first ranked the 25 by *"does it carry a defect yet"* — hand-typed EDN identity, a mirrored Rust
enum — found none, and recommended **moving zero**. Then I excluded two more for having no textual
uses. Both are the same error, and my own record names it:
*an honesty defect is not gated on demand.* **"Nobody has tripped over it" is precisely what a
silent drift surface produces.** The criterion is categorical: a type wat uses is declared in wat.

## THE SURFACE — measured 2026-09-06

```
25  enums registered as hand-written Rust literals in src/types.rs
23  of them used by wat code            RecvOutcome 1798 sites · ConnectOutcome 892 · SendOutcome 801
 0  declared in BOTH places             — no reconciliation, nothing to diff
 9  parametric                          — UNREGISTERABLE FROM WAT until H-3 landed hours ago
97  Rust string-literal sites total      max 10 for any single enum
1247 doc lines of variant prose that live only in Rust, unreachable from `metadata-of`
```

The two with no textual use (`CloseOutcome`, `DegenerateSide`) are IN, not out. A registered type
nothing references is either reachable in a way a grep cannot see or it is dead — and declaring it
is what tells us which. Skipping it preserves the ambiguity; if the move surfaces that it is dead,
that is a `purgare` finding and a gift, not a reason to have skipped it.

## ⛔ AND THE SWEEP ALONE BUYS NOTHING PERMANENT

```
tests/lint/ — NOTHING refuses a hand-written TypeDef::Enum(EnumDef { … }) literal.
```

The 26th can be hand-written tomorrow. **The wall is the stone; the sweep is what makes the wall
landable.** Without it this is 25 fixes and an open door — the exact "fix the instance, leave the
class" shape the whole arc is against.

★ The wall is cheap because the substrate already enforces the hard half: a wat-sourced type is
re-declared by the corpus at load, and `Existing::Equivalent` no-ops **only if byte-equivalent** —
`tests/lint/wat_record_from_sources_are_loaded.rs` says it outright: *"Disagree, and the stdlib
refuses to load outright."* The lint only has to refuse the hand-written FORM at the door.

## THE ONE CONTRACT DECISION — homes

**One file per namespace family, named for the family, holding that family's type declarations.**

```
:wat::core::    2   ->  wat/core.wat            EXISTS (H-3 already put Option/Result there)
:wat::holon::   5   ->  wat/holon.wat           EXISTS
:wat::kernel:: 11   ->  wat/kernel/outcomes.wat NEW — all eleven are outcome types; the three that
                                                any wat file references are referenced from a
                                                SERVICE IMPL (kernel/services/stdio.wat), which is
                                                a consumer, not a home
:wat::edn::     3   ->  wat/edn.wat             NEW
:wat::eval::    3   ->  wat/eval.wat            NEW
:wat::stream::  1   ->  wat/stream.wat          NEW
```

A file MAY host a cross-namespace type — `wat/core.wat:2137` declares `:wat::kernel::Location`
today — so this is a judgment, not a rule the substrate enforces. Four new files is not waste: it
is the shape `wat/kernel/diagnostics.wat` already set, and it puts 1247 lines of prose where the
corpus can read them.

**Every new file MUST join `STDLIB_FILES`.** `tests/lint/wat_record_from_sources_are_loaded.rs`
already gates exactly this and will go red if one is missed — a gate we get for free.

## WHY THE ORDER IS SWEEP-THEN-WALL, IN ONE STRIKE

H-1 landed its wall alone because it was **observationally inert** (nothing in the corpus violated
it). This wall is the opposite: it goes red on 25 sites the moment it exists. So it lands *after*
the sweep — **in the same strike**, because a wall deferred to "a follow-up" is the deferral this
arc's own discipline rejects. Done is done.

## FOUR QUESTIONS

| | |
|---|---|
| **Obvious?** | YES — "a type wat uses is declared in wat" needs no exception list |
| **Simple?** | YES — one `defenum` + one `wat_enum_register_from!` per type; the mechanism is complete as of H-3 |
| **Honest?** | YES — it removes the class rather than the instances that happened to bite, and the 1247 prose lines stop being invisible to the language that owns the types |
| **Good UX?** | YES — `metadata-of` and the doc row reach every one of these types for the first time |

## OUT OF SCOPE, AFFIRMATIVELY

Records and structs still registered as Rust literals (the same class, a sibling stone — this one is
enums, so a red is never ambiguous between the two) · the match arm · the constructor form.
