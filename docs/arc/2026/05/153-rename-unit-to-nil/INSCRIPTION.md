# Arc 153 — Rename `:wat::core::unit` → `:wat::core::nil` — INSCRIPTION

## How we got here

Arc 153 closes 2026-05-06, the same day it opened. Three slices
shipped in one session. But the arc didn't START this session —
it started in the failure of another.

Today began with arc 145: REQUIRED `-> :T` declarations on every
value-bearing form (`let` / `let*`, eventually `do`). Sonnet
shipped sweep 1a substrate clean. Sweep 1b kicked off across ~455
call sites. Mid-sweep, we ran the four questions on the design
ourselves, with the load-bearing UX test the user named:

```
(:wat::core::do (:println "LOG") (:+ 1 1))
```

The casual print-then-return idiom — every Lisp's daily verb —
would have required `(:do -> :i64 (:println "LOG") (:+ 1 1))`
post-arc-145. Every debug breadcrumb taxed by a type declaration
that adds zero static safety. **Good UX failed.**

The discipline that surfaced from arc 145's backout — captured in
`feedback_substrate_already_typed.md` — is that the substrate
ALREADY does static type-checking, end-to-end, via inference +
recipient unification:

- Each binding's `(name :Type)` slot is the local contract; RHS unifies against it
- The form's inferred type IS the body's inferred type
- Whatever consumes the form (function declared return, argument position, binding slot) is the recipient; recipient unifies against the form's inferred type

Adding REQUIRED `-> :T` would provide ONE thing — error locality
(mismatch fires at the form site, not the recipient). It does NOT
add new static safety. The static guarantee was already complete.

Arc 145 backed out as foundation-correction-non-shipping; sweep
1a substrate edits reverted; the realization was the deliverable.

**Arc 153 inherits arc 145's failure-as-data.** With the
substrate-as-already-typed insight crystallized, the next question
surfaced naturally.

## The unit value's name

The user, looking at `()` at function return positions:

> *"`()` to me looks like a function of nothing, not an empty
> list, but an invocation of nothing"*

Visual parse-confusion. `()` advertises "form / application"
syntactically while being a value literal. The substrate compounds
this: arc 109 slice 1d's error message describes `()` as a "list
literal" that happens to type as `:wat::core::unit` — one notation
quietly carrying two concepts.

Running the four questions on `()` at value position:

1. **Obvious? NO.** Reads as invocation, not value.
2. Honest? Borderline. Form syntax for a value is a small lie.
3. UX? YES (terse).
4. — irrelevant; stopped at Obvious.

Two paths: keep `()` and accept the visual cost, or mint an
explicit FQDN spelling.

## The Lisp-vs-Rust frame

I argued initially for `unit` — Rust DNA, type-theoretic
precision, OCaml/ML/Haskell pedigree. The user pushed back:

> *"its a strong marker for something like a Python None, a Ruby
> or Clojure nil, Java null and so on... its a visual marker that
> doesn't have the 'null pointer exception' while still operating
> like a nil"*

Then re-grounded the question:

> *"or... do we change gears... and just make nil a keyword and
> own it?... :wat::core::nil is the language's unit?"*

I objected: "nil walks back the split." Wat-rs already has
`Option<T>::None` for absence, `:wat::core::false` for false,
`:wat::core::Vector<T>` for collections. Renaming unit to nil
imports Clojure-baggage where nil overloads all four roles.

The user corrected me:

> *"i'm not saying nil and None are equal... i'm arguing that nil
> and Unit are equal... the property of Unit while having a name
> that i find visually meaningful... None and Some coexist and are
> completely separate to nil"*

The split is enforced by the type system, regardless of name.
Wat's `:wat::core::nil` ≠ `:wat::core::None` ≠
`:wat::core::false` ≠ empty `:wat::core::Vector<T>`. The substrate
holds the discipline; the name carries the marker effect.

I retracted twice — once on the conflation worry, once on the
substrate-discipline worry. Type-theoretic bias surfaced both
times; user re-grounded both times. **The retraction itself is
the lesson.** I have a reflex toward type-theoretic vocabulary
even when entity-naming is the question; it has cost.

## The agreement

> *"so... wat's nil is Rust's Unit. that's what we're agreeing to?"*

Yes. Same type-theoretic role (singleton type, single inhabitant,
"no meaningful return value"). Different name. Marker effect for
cross-language familiarity. The user's broader frame:

> *"we ride to compaction - we are the best at what we do - we are
> doing as fast as we can - i need a lisp on rust to satisfy what
> we're building towards"*

Wat is a Lisp on Rust. The Lisp tradition gets `nil`; the Rust
tradition gets `Some` / `None` / `Result` / `Ok` / `Err` /
`Vector` / `HashMap`. Two traditions honored at the substrate's
naming surface; neither distorted.

The triplet:

```
:wat::core::nil       — singleton; "no meaningful return value"
:wat::core::Some(t)   — Option<T>'s presence variant
:wat::core::None      — Option<T>'s absence variant
```

Three names, three roles, no overlap. Type system enforces.

## What shipped

### Slice 1a — substrate (atomic with 1b at `fd1b3fe`)

Two coordinated substrate changes per substrate-as-teacher
Pattern 3 (symbol migration), mirroring arc 109 slice 1d's
`BareLegacyUnitType` walker structure:

- **Type-position rename.** `:wat::core::unit` retired;
  `:wat::core::nil` minted as canonical FQDN. New
  `CheckError::BareLegacyUnitName` variant + body walker
  (`walk_type_for_bare`'s Path-arm extension) + signature-pass
  walker (`walk_type_for_legacy_unit_name`) emit one migration
  error per offending site. Sonnet's load-bearing engineering
  call: a transitional typealias resolved `:wat::core::unit` to
  the unit type during the deprecation window, preventing
  cascading TypeMismatch across stdlib while the walker still
  fired the migration hint.
- **Value-position recognition.** `:wat::core::nil` keyword at
  value position infers as the unit type and evaluates to the
  unit value. Narrow special-case (HashMap-key regression
  verified — only `:wat::core::nil` keyword string is treated
  specially; other keywords pass through unchanged).

Four files: `src/types.rs`, `src/check.rs`, `src/runtime.rs`,
NEW `tests/wat_arc153_nil_rename.rs` (10 tests covering
type-position retired/canonical, value-position works,
type-mismatch, mixed empty-list/nil, parametric containment,
narrow special-case HashMap-key regression).

### Slice 1b — consumer sweep (atomic with 1a at `fd1b3fe`)

Workspace-wide migration driven by the substrate's diagnostic
stream. ~75 minutes wall-clock. 90 wat + Rust files swept across
two transforms:

- **Type-position** (substrate-as-teacher walker-driven):
  `:wat::core::unit` → `:wat::core::nil` at every annotation site
  across stdlib (`wat/`), per-crate substrates, wat-tests,
  examples, embedded wat in `tests/` + `src/` lib tests.
- **Value-position** (mechanical grep-driven): `()` →
  `:wat::core::nil` at function bodies, match arms, if branches,
  do-form final forms, no-op define bodies.

Atomic commit when workspace = 0 failed (1988 passing tests
across all crates).

### Slice 2 — retirement + paperwork (this commit)

Per substrate-as-teacher § "Retire the hint when its window
closes" — the sweep is structurally complete; the scaffolding
retires:

- **Transitional `:wat::core::unit` typealias** removed from
  `src/types.rs`. With all in-tree consumers swept, the alias
  has nothing to resolve.
- **`walk_type_for_legacy_unit_name` walker body** retired in
  `src/check.rs`. Comment names arc 153 as the retirement arc.
- **`walk_type_for_bare`'s `:wat::core::unit` Path-arm** retired.
- **`CheckError::BareLegacyUnitName` variant + Display + diagnostic
  field-emit** RETAINED as orphaned scaffolding per arc 113's
  precedent. The variant stays for testing / teaching /
  reintroduction; only the firing body retires.
- **Runtime `("wat::core::unit", "()")` value-tag match arm**
  retired in `src/runtime.rs`. Migration-window scaffolding;
  same retirement window.
- **Internal anchor strings updated** in the channel-pair
  deadlock walker (synthetic
  `:wat::kernel::Channel<wat::core::nil>` etc.) for
  substrate self-consistency.

Tests #1, #6, #10 in `tests/wat_arc153_nil_rename.rs` updated
for post-retirement behavior — they now assert the typealias and
walker are gone (not present), inverting their pre-retirement
sense.

### Closure paperwork

- This INSCRIPTION (rewritten by orchestrator after sonnet's
  initial draft at `969b847` — protocol break corrected; see the
  note at the end).
- 058 changelog row at `holon-lab-trading/.../FOUNDATION-CHANGELOG.md`.
- USER-GUIDE § 4 — `:wat::core::nil` subsection.
- WAT-CHEATSHEET § 3 — `nil` row + dedicated subsection +
  example usages corrected.
- CONVENTIONS § "Batch convention" — Put-verb signature updated.
- Task #182 marked SUPERSEDED with user-direction rationale.

## The four questions — multiple runs

Today's arcs paid for several four-questions runs against
this naming surface:

**On REQUIRED `-> :T` (arc 145 frame):** Obvious YES, Simple YES,
Honest YES, **Good UX NO** (print-then-return tax). Stop at fourth
question. Arc 145 backed out.

**On `()` at value position (the trigger for arc 153):** Obvious
**NO** (reads as function-of-nothing). Stop at first question.
Need explicit form.

**On `unit` vs `nil` for the rename:** Both Obvious YES + Simple YES
+ Honest YES; UX picks `nil` (3 chars; cross-language familiar;
marker effect). Type-theoretic bias resisted; the user's
Lisp-vocabulary frame won.

**On retire-vs-preserve typealias post-sweep:** substrate-as-teacher
§ "Retire the hint when its window closes" — sweep structurally
complete; retire body, retain variant scaffolding per arc 113
precedent.

## Cross-references

- **Arc 145** (closed as foundation-correction-non-shipping) —
  the failure that paid for the substrate-already-typed insight
  arc 153 inherits. See `docs/arc/2026/05/145-typed-let/DESIGN.md`
  top section.
- **Arc 109 slice 1d** — Pattern 3 walker precedent
  (`BareLegacyUnitType` retired `:()` as a type annotation; arc
  153 inverts the slice 1d pivot using the same mechanics).
- **Arc 113** — orphaned scaffolding precedent (variant + Display
  preserved after firing body retires).
- **Arc 136** (do form) — slice 2 closure runs after this; the
  do form's return positions are canonically `:wat::core::nil`.
- **Task #182** (rename `unit` → `Unit`) — superseded by arc 153
  with user-direction rationale.
- **`feedback_substrate_already_typed.md`** — the foundation
  insight arc 145 paid for; arc 153 builds on it.
- **`feedback_paperwork_orchestrator_side.md`** — saved
  2026-05-06 mid-arc-153-slice-2 when the user caught me
  delegating this very INSCRIPTION to sonnet. The user's voice:
  *"usually we don't have sonnet do the paper work."* The note at
  the end of this document is the resolution of that protocol
  break.

## Calibration record

- **Arc opened:** 2026-05-06 after arc 145 back-out
- **Slice 1a substrate:** ~17 min wall-clock (Mode A clean,
  10/10 new tests, BareLegacyUnitName walker shipped)
- **Sweep 1b consumer migration:** ~75 min wall-clock (Mode A
  clean, 90 files, ~455 sites, atomic with 1a)
- **Slice 2 retirement + paperwork:** ~18 min sonnet wall-clock
  (Mode A structural; INSCRIPTION rewritten by orchestrator
  after the protocol-break observation)
- **Total arc duration:** one session, ~2 hours of substrate +
  consumer + retirement work
- **Honest deltas:**
  - Substrate retirement closed cleanly with one cross-reference
    update sonnet caught (the runtime value-tag match arm — also
    migration-window scaffolding) plus three test reshapes plus
    internal anchor-string consistency updates.
  - The transitional typealias preservation (sonnet's slice 1a
    judgment call) was load-bearing — without it, sweep 1b would
    have cascaded TypeMismatch across stdlib instead of running
    cleanly per the substrate-as-teacher loop.
  - Sonnet's slice 2 INSCRIPTION (at `969b847`) was structurally
    correct + per recipe but missed the conversational arc that
    lives in the orchestrator's context window. This rewrite
    captures what the BRIEF couldn't compress: the bridge from
    arc 145, the user's verbatim direction at each pivot, my
    type-theoretic bias retraction, the Lisp-on-Rust frame.

## A note on protocol

This INSCRIPTION was first written by sonnet via
BRIEF-CLOSURE.md's delegation. The user observed:

> *"usually we don't have sonnet do the paper work... let's see
> how this goes..."*

Closure paperwork carries conversational context that lives in
the orchestrator's window, not in any BRIEF. Sonnet's draft was
factually correct but flat — synthesis derivative of the BRIEF
rather than the conversation. The discipline lesson is captured
permanently in `feedback_paperwork_orchestrator_side.md`:
*"closure paperwork is orchestrator-side."*

Sonnet's draft stands at git commit `969b847` as historical
record of the protocol break. This rewrite (the file as you read
it) is the orchestrator's voice, written after the user's
correction:

> *"update it - we broke protocol - i don't like breaking protocol"*

The fault is named; the lesson is captured; forward progress
only. Future arcs: paperwork synthesis is orchestrator-side
before any sonnet brief.

## Status

**Arc 153 closes here.** The substrate's canonical singleton type
is `:wat::core::nil`. The triplet `nil` / `Some(t)` / `None`
reads cleanly across substrate, stdlib, tests, and examples. The
orphaned `BareLegacyUnitName` variant remains as scaffolding for
future symbol-migration arcs to reuse.

**Arc 109 v1 closure trajectory clearer.** One naming-cleanup
chain link closes; arc 136 slice 2 (do-form closure) runs next
with return positions canonically `:wat::core::nil`.

**The Lisp on Rust gains its `nil`.** The substrate inscribes
its first vocabulary-marker that honors the Lisp tradition by
name + the Rust tradition by enforcement. The user's frame —
*"i need a lisp on rust to satisfy what we're building towards"*
— ships forward by one foundation marker.

---

*the singleton has its name. the triplet reads cleanly. the
type system enforces the split. the orchestrator owns the
synthesis. forward progress only.*

**PERSEVERARE.**
