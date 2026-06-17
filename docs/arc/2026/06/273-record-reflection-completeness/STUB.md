# STUB — Arc 273: record reflection completeness (value `record?` + a type-level record predicate)

> Stubbed 2026-06-16 (builder: *"we should have a record? and holon-record? or something? … we just
> covered a deeper issue … stub an arc to go deal with this … not a now thing unless it becomes one"*).
> NOT a now-thing. Surfaced building arc-272 record-state rs-1 (the defservice `:state`-must-be-a-record
> check). Grounded against HEAD `c3cf58b5`.

## The two gaps

**(1) `record?` is incomplete — it only recognizes HOLON records.**
`:wat::core::record?` (`runtime.rs:3860`, arc-234.3a) is `∀T. T -> bool — true iff input is
Value::wat__holon__Record`. But there are TWO record flavors on the wire (the record-vs-struct law):
- base `wat__Record` (tagged map, named fields)
- holon `wat__holon__Record` (rides `holon_form`-as-edn)
A **base** record handed to `record?` returns **false** — a lie (it IS a record). The predicate should be
complete: `record?` = "is this value EITHER record flavor"; with `holon-record?` (and/or `base-record?`)
for the flavor distinction when a caller genuinely needs it. (intueri the names when built.)

**(2) There is no TYPE-level record predicate.** `record?` is a VALUE predicate (does this *value* carry a
record). rs-1 needs a different axis: at defservice expansion, given the `:state` **type keyword** (e.g.
`:my::CounterState`), decide whether it names a **registered record type** — to reject a scalar/struct
`:state`. Candidates to weigh when built:
- macro-time `(subtype? :T :wat::Record)` — UNPROVEN that `subtype?` resolves against the type registry at
  EXPAND time (the record must be registered before defservice expands; and the macro-eval engine must
  reach the registry). Probe it.
- a new `record-type?` reflection (type-keyword → bool) on the macro purity fence + the checker.

## What this blocks (the trigger)

arc-272 **record-state rs-1** — the defservice CHECK that `:state` must be a record
(`DESIGN-STONE-record-state-final-return.md`, `NOTE-service-final-state-return.md`). rs-1 is DEFERRED onto
this arc. The record-state FEATURE (rs-2 serve-returns-final-state + thread await; rs-3 process await)
does NOT depend on this — it works with any EDN-serializable state — so the feature can proceed without
the strict check; only the *guard* waits here.

## Bar

Build when it becomes a now-thing (rs-1 needs it, or a `record?` caller hits the holon-only lie). Then:
fix `record?` to cover both flavors (+ flavor predicates), and add/prove the type-level record predicate
for macro-time checks. Pairs `NOTE-service-final-state-return.md` + DESIGN-STONE-record-state-final-return
+ the record-vs-struct law (`wat/spawn.wat:116`) + [[feedback_no_magic_that_lets_llm_fake_correctness]].
