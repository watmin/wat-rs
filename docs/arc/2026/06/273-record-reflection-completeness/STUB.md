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

## What this blocks (the trigger) — CORRECTED 2026-06-16: rs-1 is NOT blocked here

The original stub said arc-272 **rs-1** (the defservice `:state`-must-be-a-record CHECK) was DEFERRED onto
this arc. **That was wrong** — it conflated gap (1) with gap (2). rs-1 needs only the **TYPE-level** check,
and that machinery ALREADY EXISTS at CHECK time: `src/collection/infer.rs:378-381` =
`is_subtype(ty, ":wat::Record", env) || is_subtype(ty, ":wat::holon::Record", env)` over `subtype_edges`
(`TypeDef::Record` first-class, Stone S-B.1; `env.types().get`). So rs-1 was promoted to a HARD REQUIREMENT
(builder 2026-06-16) and builds NOW against that, NOT here. rs-2 SHIPPED (the `:Stop` op); rs-3 REJECTED.

Gap (2)'s *macro-time* `subtype?`/`record-type?` is only relevant IF rs-1's design picks the macro-expand
route over the check-time route — an rs-1 design call, not a 273 dependency. **This arc (273) now carries
ONLY gap (1):** the runtime VALUE predicate `record?` is holon-only (a lie for base records) — genuinely
independent of rs-1, still NOT a now-thing until a `record?` caller hits the holon-only lie.

## Bar

Build when it becomes a now-thing (rs-1 needs it, or a `record?` caller hits the holon-only lie). Then:
fix `record?` to cover both flavors (+ flavor predicates), and add/prove the type-level record predicate
for macro-time checks. Pairs `NOTE-service-final-state-return.md` + DESIGN-STONE-record-state-final-return
+ the record-vs-struct law (`wat/spawn.wat:116`) + [[feedback_no_magic_that_lets_llm_fake_correctness]].
