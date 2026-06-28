# BRIEF — 293.4d: a field member is an accessor too → the acceptance demo GREEN (R1 FORMA SOLA SUFFICIT)

**The work, in one paragraph.** 293.4b/c made METHOD members dispatch and be satisfiable. But a FIELD member called
through the surface — `(:Surface/color s)` — is `UnknownCallee` (the dispatcher only recognizes method members), and a
FIELD member is satisfiable only by a struct field (so a foreign type backing it with a method can't satisfy). This is
the LAST seam of "methods are accessors": **every surface member — field OR method — is an accessor `:T/name`.** Make
the resolve / check / dispatch layers treat a field member exactly like a method member (route `:Surface/name s` →
`:<T>/name`), and make satisfaction accept a field member backed by EITHER a struct field OR a `:T/name` accessor
(method/extend). Then fix the acceptance demo's one bug and un-ignore it = the arc's GREEN gate, R1 fulfilled.
**NO `defprotocol` touch — that annihilation is the separate 293.4e strike.**

## The one contract decision (pinned)
A surface member (Field or Method) is an accessor `:T/name`. `(:Surface/name s)` dispatches by `s`'s runtime type to
`:<T>/name` — for a Field member, `:<T>/name` is the record's auto-generated field accessor OR (foreign) an extend
method; for a Method member, the `defn`/extend method (293.4b/c). Satisfaction: member satisfied iff `:<T>/name`
resolves with an assignable type — a Field member additionally satisfied by an actual struct field of the right type
(records). The field-vs-method distinction is invisible at the surface, the call site, and the satisfier.

## The two RED gates (committed)
- `tests/types/probe_arc293_4d_field_member_accessor.{rs,wat}` (FOCUSED, `#[ignore]`'d RED) — a `:t::Colored` surface
  with a FIELD member `color`; `(:t::Colored/color (:t::Ball …))` must dispatch to `:t::Ball/color` (the record's field
  accessor) → "red". Verified RED: `UnknownCallee`. No extend/Vector — isolates the field-member dispatch.
- `tests/types/probe_arc293_acceptance_demo.{rs,wat}` (COMPREHENSIVE, the arc's gate, `#[ignore]`'d RED) — Shape/Circle/
  Square + the holon-Vector monkeypatch. Verified RED with FOUR errors: `:geo::Shape/{color,label,area}` UnknownCallee
  (field+method members not all dispatched) + the Vector `TypeMismatch` (see the demo bug below).

## Read in order (the rooms — grounded 2026-06-28)
1. **`src/resolve/walk.rs`** (293.4b's `is_resolvable_call_head` surface arm) — it accepts `:S/m` when `m` is a METHOD
   member; broaden to accept `m` being ANY member (Field or Method) of the surface.
2. **`src/check.rs:5789` neighborhood** (293.4b's surface-method call-site check) — broaden: `:Surface/name` where
   `name` is a Field member types as the field's `TypeExpr` (the accessor returns the field type); a Method member as
   today. Receiver satisfies the surface.
3. **`src/runtime.rs` ~5300** (293.4b's surface-method dispatch arm) — broaden the member-match from `SurfaceMember::
   Method` only to Field-or-Method; the routing (`sym.get(canonical_callable_name(":<T>/<name>")) → apply_function`) is
   UNCHANGED — a record's field accessor `:<T>/<field>` is already a `sym.functions` entry, so the same lookup works for
   both. (Confirm a record's field accessor is registered under `:<T>/<field>` — grep how defrecord registers accessors.)
4. **`src/check.rs:14380` + `src/types/surface.rs` (`struct_satisfies_surface`)** — a Field member is satisfied by a
   struct field (today) OR by a `:<T>/name` accessor resolving (the 293.4c `resolve_method` path). UNIFY: a Field member
   is satisfied iff (the candidate has a struct field `name` of an assignable type) OR (`resolve_method(":<T>/name")`
   returns an assignable accessor). This lets the foreign Vector satisfy `:geo::Shape`'s `color` FIELD member with its
   `color` extend METHOD. Keep the foreign-type bound (no holder) from 293.4c.
5. **`tests/types/probe_arc293_acceptance_demo.wat`** — FIX THE DEMO BUG: it does
   `(:wat::core::extend-type :wat::holon::Vector :geo::Shape …)` but constructs `(:wat::core::Vector :wat::core::i64 …)`,
   which produces a `Value::Vec` whose `type_name` is `:wat::core::Vector` (NOT `:wat::holon::Vector` — that is a
   different `Value::Vector` variant). The extend target must MATCH the constructed value's type → change the extend
   target to **`:wat::core::Vector`**. (Grounded: `eval_vector_ctor` → `Value::Vec`; `type_name(Value::Vec)` →
   `wat::core::Vector`.) Do NOT change the construction; the demo's `area` uses `(:wat::core::length self)` which works
   on a `Value::Vec`.

## Implementation sketch
- resolve/check/runtime: change the surface member-match from "is a Method member" to "is a member (Field|Method)".
  The dispatch + the `:<T>/name` lookup are unchanged — records register `:<T>/field` accessors already.
- satisfaction: Field member satisfied by struct-field OR `:<T>/name` accessor (union); Method member as 293.4a/c.
- demo: `:wat::holon::Vector` → `:wat::core::Vector` in the extend-type head. Un-ignore both probes.

## Blast radius (bounded)
`src/resolve/walk.rs`, `src/check.rs` (the call arm + the satisfaction), `src/runtime.rs` (the dispatch arm),
`src/types/surface.rs` (the satisfaction helper), the demo `.wat` (1 keyword). NO `defprotocol` touch. NO change to
293.4a/b/c's method-member paths beyond broadening the match.

## STOP triggers (halt + surface; do NOT improvise)
- **STOP-1 (record field accessor not at `:<T>/field`):** if a record's field accessor is NOT registered under
  `:<T>/<field>` in `sym.functions` (so the dispatch `:<T>/name` lookup misses for a field member) — STOP and report
  where field accessors live; do not invent a parallel lookup.
- **STOP-2 (the demo needs more than the field-symmetry + the extend-target fix):** if, after parts 1–5, the demo still
  REDs on something NOT covered here (e.g. `:wat::core::length` on a `Value::Vec`, `i64::to-f64`, the `str` concat) —
  STOP and report the exact remaining error; do not patch around it.
- **STOP-3 (satisfaction goes always-true):** broadening the Field-member satisfaction must not make every type satisfy
  every surface. A type with neither the struct field NOR a `:<T>/name` accessor must still be rejected. The focused
  probe + the 293.4c `_notextended_bad` negative must both still pass.

## EXPECTATIONS (the gate)
| # | what | command | expected |
|---|---|---|---|
| 1 | the focused field-member probe GREEN | `cargo nextest run --release -E 'test(field_member_dispatches_through_the_surface)'` | PASS ("red") |
| 2 | THE ACCEPTANCE DEMO GREEN (un-ignored) | `cargo nextest run --release -E 'test(shape_demo_fields_and_methods_and_the_monkeypatch)'` | PASS — R1 FORMA SOLA SUFFICIT |
| 3 | 293.4a/b/c un-regressed | `cargo nextest run --release -E 'test(method_member) + test(surface_method_dispatches) + test(extend_type_teaches)'` | all PASS |
| 4 | satisfaction still bounded (negatives) | the 293.4c `_notextended_bad` + the focused probe's implicit negative | reject |
| 5 | whole workspace green | `cargo nextest run --release` | floor 0 (4093+ passed) |

## You are a LEAF
Anchor cwd `/home/watmin/work/holon/wat-rs`; `pwd` first; reject any `.claude/worktrees/` path. Do NOT spawn subagents.
Do NOT commit. Build incrementally. Read every diff end-to-end. Self-verify the EXPECTATIONS. If a STOP fires or the
work exceeds the brief, halt and report.
