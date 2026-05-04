# Arc 109 — Deferral violations tracker

**Created:** 2026-05-03 evening, after the user surfaced the closure-
discipline violation:

> *"i was reviewing some of the arc files and found us saying stuff
> like 'we'll do this later' AND MARKED IT INSCRIBED -- the
> inscription is our 'done-ness' measurement... if there's a
> deferall - the arc stays open until all deferrals are done...
> we need a new doc in 109 tracking violations to this.... it was
> very disappointing to see that we have arc being marked as done
> with pending work still..."*

## The principle

**INSCRIPTION = DONE.** An arc with an INSCRIPTION.md is closed.
Closed means every commitment the DESIGN made has shipped. If
ANY deferral remains — "we'll do this later" / "future arc when X
surfaces" / "deferred to a small follow-up" / "TODO" — the arc
**is not done**. It must stay open until the deferred work
either ships in a follow-up arc OR is removed from scope (the
scope reduction itself documented).

**No soft "honest scope" exceptions.** "Future arc if a caller
needs" is still a deferral; the arc claims completion while
naming pending work; that's the violation pattern. Either the
"if a caller needs" work is in scope (ship it before INSCRIPTION)
or it's out of scope (DESIGN must say "out of scope; not
tracked"). Anything in between is the bug.

## Why this matters

The user has been chasing the "impeccable foundation" milestone
for over a week. Every deferral that ships under the cover of
"INSCRIBED" is a foundation crack hidden behind a closure
declaration. The cumulative effect: the foundation looks
complete on the changelog (N arcs INSCRIBED) while carrying
hidden debt the next-leg work will trip on. Arc 109 v1 closure
is the worst place to discover this — the user named the
discipline violation explicitly so the next-leg work doesn't
inherit shaky ground.

## Audit method

A research agent crawled `docs/arc/2026/04/` + `docs/arc/2026/05/`
on 2026-05-03 evening, reading every INSCRIPTION.md + DESIGN.md
for deferral language: "deferred", "future arc", "future fix",
"later", "out of scope", "NOT in this arc", "follow-up", "TODO",
"scratch", "punted", "next arc", and similar.

**The audit is a SAMPLE, not exhaustive.** ~60+ deferrals were
found across ~70 inscribed arcs. The audit named the most
egregious violations (INSCRIPTION + multiple deferrals + no
named follow-up) but a complete enumeration likely surfaces more
on closer reading. **Treat this tracker as the starting point
for remediation, not the complete count.**

## Violations — arcs marked INSCRIBED with open deferrals

### Arc 016 — Failure location and frames

**INSCRIPTION date:** 2026-04-21
**Path:** `docs/arc/2026/04/016-failure-location-frames/INSCRIPTION.md`

Five deferrals named explicitly in the INSCRIPTION:

- **Line 260-261**: *"Color output... Deferred — ASCII works; color is polish."*
- **Line 267**: *"Hook composability chain introspection... If a consumer surfaces need, a future arc can chain hooks."*
- **Line 269-271**: *"pytest-style value substitution... Future arc if a test author demands it."*
- **Line 272-275**: *"Hermetic-fork propagation... A future arc could channel structured frames back via a sidecar pipe if demand surfaces."*
- **Line 277-279**: *"Parse / check / resolve diagnostic span unification... A future arc could unify."*

**Status:** None of these have shipped as named follow-ups.
INSCRIPTION claims "complete" on line 325 with five pending items
in the open-items list.

**Remediation needed:** EITHER spawn five follow-up arcs (or one
consolidated arc) that ships these items, OR revise the
INSCRIPTION to scope-bound them as "out of scope; not part of
arc 016" with an architectural justification.

---

### Arc 017 — Loader option for consumer macros

**INSCRIPTION date:** 2026-04-22
**Path:** `docs/arc/2026/04/017-loader-option-consumer-macros/INSCRIPTION.md`

Five deferrals named:

- **Line 175-179**: *"Expression-shaped loader argument... Deferred."*
- **Line 180-182**: *"Per-test loader differentiation... Different loaders = different tests/*.rs files."*
- **Line 183-186**: *"Library-file error attribution... A future polish could distinguish."*
- **Line 187**: *"FsLoader (unrestricted) as a macro option. Deferred."*
- **Line 188-190**: *"Sandbox bodies inheriting outer-file (load!)'d defines... If a concrete caller asks, that's a separate arc."*

**Status:** No follow-up arcs named or shipped. INSCRIPTION claims
completion while explicitly listing five pending items.

**Remediation needed:** Same shape as arc 016. Ship or scope-bound
each item.

---

### Arc 048 — User-defined enum value support

**INSCRIPTION date:** 2026-04-24
**Path:** `docs/arc/2026/04/048-user-defined-enums/INSCRIPTION.md`

Six deferrals named, including one CRITICAL violation:

- **Line 248-249**: *"Variants with named fields... Add when a caller needs."*
- **Line 250-251**: *"Generic user enums... Open its own arc if needed."*
- **Line 252-253**: ⚠️ *"USER-GUIDE Forms appendix sync. Deferred to a small follow-up."*
- **Line 254-255**: *"Migrate Option/Result to Value::Enum. Substantial sweep with no semantic gain."*
- **Line 256-257**: *"Atom-of-enum integration tests... Explicit tests can land when a caller needs them."*
- **Line 258-262**: *"Sweep of any potential conflicts... They coexist — no collision."*

**Status:** Line 253 explicitly says "deferred to a small follow-up
arc" — a follow-up that never shipped. Documentation drift baked
into the closure declaration. **This is the clearest match to the
user's complaint pattern.**

**Remediation needed:** Ship the USER-GUIDE Forms appendix update
NOW; OR revise the INSCRIPTION to retract the deferral. Same for
the other five named items per the strict closure principle.

---

### Arc 050 — Polymorphic arithmetic

**INSCRIPTION date:** 2026-04-24
**Path:** `docs/arc/2026/04/050-polymorphic-arithmetic/INSCRIPTION.md`

Five "scope boundary" deferrals (the audit agent classified these
as "honest" but per user-strict framing they are still violations):

- **Line 226-229**: *"Polymorphic modulo `%`... Ship when a caller surfaces."*
- **Line 230-234**: *"Polymorphic `max`, `min`, `abs`, `clamp`, `round`... Add later if pressure surfaces."*
- **Line 235-240**: *"Wider integer types... if wider ints land later."*
- **Line 241-243**: *"String concatenation via `+`... wat doesn't."*
- **Line 244-246**: *"Lab sweep... migrate per-arc judgment if touched."*

**Status:** "Ship when a caller surfaces" / "add later if pressure
surfaces" is deferral language by the user's strict framing. Arc
148 (shipped today) materially expanded this surface; some of these
items may now be closed via arc 148, but the cross-references
weren't updated.

**Remediation needed:** Audit each item against arc 148's actual
ship; mark closed-by-arc-148 where applicable; revise INSCRIPTION
to either ship or out-of-scope the rest.

---

### Arc 060 — `:wat::kernel::join-result`

**INSCRIPTION date:** 2026-04-26
**Path:** `docs/arc/2026/04/060-join-result/INSCRIPTION.md`

One deferral:

- **Line 150-151**: *"Structured panic payloads... Future arc when a caller needs it."*

**Status:** Per the user-strict framing, this is a deferral.

**Remediation needed:** Either ship it or revise the INSCRIPTION
to scope-bound it as out-of-arc-060.

---

### Arc 062 — `:wat::core::Bytes` typealias

**INSCRIPTION date:** 2026-04-26
**Path:** `docs/arc/2026/04/062-bytes-typealias/INSCRIPTION.md`

One deferral:

- **Line 150**: *"6a — Hashing performance... Deferred."*

**Status:** Single small deferral; technically a violation per
strict framing.

**Remediation needed:** Same as above.

---

### Arc 085 — Enum-derived SQLite schemas

**INSCRIPTION date:** 2026-04-28
**Path:** `docs/arc/2026/04/085-enum-derived-sqlite-schemas/INSCRIPTION.md`

Four deferrals:

- **Line 133-137**: *"Unit variants emit nothing... Future arc adds an event-style table when a consumer surfaces a need."*
- **Line 138-142**: *"Option<T> fields not yet supported... Future arc adds."*
- **Line 143-148**: *"Table-name overrides... Future arc adds an annotation syntax."*
- **Line 149-150**: *"Schema migrations... First run creates tables..."* (truncated; behavior on second run not in scope)

**Status:** Four "future arc" statements in the INSCRIPTION. None
have named-and-shipped follow-up arcs.

**Remediation needed:** Same shape — ship or scope-bound.

## Near-violations caught in flight (not in INSCRIPTIONS)

### Arc 145 — Typed `let` (the typed-let DESIGN miscommunication)

**Discovered:** 2026-05-03 evening when user clarified the scope.
**Path:** `docs/arc/2026/05/145-typed-let/DESIGN.md`

The arc 145 DESIGN's Q2 originally said:

> *"Per user direction 'users can make their own choice' — `-> :T`
> is OPTIONAL forever. Backwards compatible; users adopt at their
> own pace if they want explicit value-bearing declaration."*

The orchestrator (me) misinterpreted the user's "users can make
their own choice" framing — that referred to `let` vs `let*`
(parallel vs sequential binding), NOT to optionality of `-> :T`.

User's correction:

> *"the ret val of a let statement /must be declared/ .. the
> 'user's choice' is whether or not to use let or let* -- both
> must have a ret val declared.. the let's ret val can be bound
> to something and used later - just like if, match etc"*

**Status:** DESIGN corrected 2026-05-03 evening. `-> :T` is
REQUIRED on both forms; existing call sites must be migrated
when arc 145 ships. The DESIGN-time deferral (making `-> :T`
optional) was caught BEFORE INSCRIPTION; this is the discipline
working as intended.

**Lesson:** the same anti-pattern — deferring consistency under a
"backwards compatibility" flag — would have shipped if the user
hadn't reviewed. Arc 145's near-violation is a worked example of
why DESIGN-time review for deferral language matters as much as
INSCRIPTION review.

## Honest open work — correctly uninscribed

For comparison, these arcs are correctly NOT inscribed (no
closure claim, no violation):

- **Arc 119** (HologramCacheService Put ack-tx) — in_progress;
  step 7 + closure pending
- **Arc 130** (cache services pair-by-index) — in_progress; slices
  1-3 pending
- **Arc 141** (docstrings) — DESIGN locked; impl pending
- **Arc 145** (typed let) — DESIGN locked; slices 1-2 pending
- **Arc 147** (substrate registration macro) — DESIGN; slices
  pending
- **Arc 149** (Ratio scratch) — DESIGN-only stub; future arc
- **Arc 151** (wat-macros wrapper honest message) — DESIGN-only
  stub
- **Arc 152** (local-bindings reflection) — DESIGN-only stub

These are honest open work — no false closure declaration; no
deferral violation pattern.

## Recommendation — remediation approach

For each violating arc:

1. **Read the named deferrals carefully.** Some may have already
   shipped via subsequent arcs without cross-references being
   updated (arc 050 vs arc 148 most likely).

2. **For each deferral, decide:**
   - **SHIP IT NOW**: open a slice in a remediation arc that
     closes the deferred item. Update INSCRIPTION to mark the
     deferral closed.
   - **SCOPE-BOUND IT OUT**: revise the INSCRIPTION to remove
     deferral language; replace with an explicit "out of arc N's
     scope; tracked in arc M / not tracked" statement.

3. **Treat this tracker as a forward-pressure document.** Every
   item here either gets shipped before arc 109 v1 closure OR gets
   explicitly retracted from the INSCRIPTION's open-items list.

## What this tracker does NOT do

- Doesn't recommend specific remediation timelines (user direction)
- Doesn't apologize for past violations — owns them, shows where
- Doesn't soften with "honest scope" classifications — per user-
  strict framing, every deferral is a violation
- Isn't exhaustive — the audit was time-boxed; closer reading of
  individual INSCRIPTIONs likely surfaces more

## The discipline going forward

**Pre-INSCRIPTION review checklist** (add to closure paperwork):

- [ ] Grep INSCRIPTION draft for: "deferred", "future arc", "later",
      "TODO", "if a caller", "if pressure surfaces", "out of scope"
- [ ] For each match: is the work shipped in this arc OR explicitly
      retracted from scope?
- [ ] If neither: STOP — do not commit the INSCRIPTION. The arc is
      not done.
- [ ] If retracted: rewrite the language from "deferred" to
      "out of scope; reason: ___; not tracked" or similar
      affirmative scope-bounding.

**Post-INSCRIPTION self-check** (orchestrator discipline):

- [ ] After every closure, re-read the INSCRIPTION through the
      "would the user say 'this is INSCRIBED but says we'll do
      this later' here?" lens.
- [ ] If yes: close the deferral immediately or amend the
      INSCRIPTION.

## Status

**Tracker active. Remediation pending user direction.** This doc
itself is a deferral-shaped artifact (it names work to do without
doing it); per the principle, the work to remediate the named
violations should be tracked by either: (a) one consolidated
remediation arc, OR (b) per-arc revisits.

The principle stands: **INSCRIPTION = DONE.** Anything else is
the violation we just named.
